#![allow(clippy::needless_return)]
#![allow(clippy::print_literal)]
#![allow(clippy::vec_init_then_push)]

use clap::{builder::{ArgAction, RangedU64ValueParser, ValueParser}, Arg, Command, ValueHint};
use fancy_regex::{Match, Regex};
use std::{env, fs, path::{Path, PathBuf}, str::FromStr};
use std::collections::HashSet;
use walkdir::{DirEntry, IntoIter, WalkDir};
use colored::{Color, ColoredString, Colorize, Styles};
use unescape::unescape;

//TODO: check all lifetime specifiers
//TODO: check all unwrap / expect usages
//TODO: write own character escaper function
//TODO: sort found paths consistently

#[derive(Debug)]
struct Args {
    paths: Vec<PathBuf>,
    patterns: Vec<Regex>,
    output_file: Option<PathBuf>,
    output_format: String,
    context: usize,
    include_hidden: bool,
    follow_links: bool,
    max_depth: Option<usize>,
}

#[derive(Debug)]
struct FoundPattern<'a> {
    pattern: &'a Regex,
    start: usize,
    end: usize
}

#[derive(Debug)]
struct FileFoundPatterns<'a> {
    file: &'a Path,
    contents: String,
    found_patterns: Vec<FoundPattern<'a>>
}

#[derive(Debug)]
struct OutputValues<'a> {
    context_before: String,
    before: String,
    matched: String,
    after: String,
    context_after: String,
    is_first_pattern: bool,
    pattern: &'a Regex,
    x: String,
    y: String,
}

fn main() {
    let args: Args = get_args();
    // println!("{:?}", args);

    //TODO: verification?

    let files: HashSet<PathBuf> = get_all_files(&args);
    // println!("{:?}", files);

    let all_found_patterns: Vec<FileFoundPatterns> = 
        find_pattern_in_files(&files, &args.patterns);
    // println!("{:?}", all_found_patterns);

    for file_found_patterns in all_found_patterns {
        generate_output_for_file(&file_found_patterns, &args);
    }
}

fn get_args() -> Args {
    let matches = Command::new("todo").author("Inferno214221").version("0.1.0")
        .about("A CLI program to find all instances of TODO notes within a file or directory")
        .disable_version_flag(true)
        .arg(
            Arg::new("VERSION").help("Print version").short('v').long("version")
                .action(ArgAction::Version)
        )
        .arg(
            Arg::new("PATH").help("TODO").action(ArgAction::Append)
                .default_value(
                    env::current_dir().expect("Should be able to get the cwd").into_os_string()
                ).value_parser(ValueParser::path_buf()).value_hint(ValueHint::AnyPath)
                //TODO: can be invalid
        )
        .next_help_heading("File Selection")
        .arg(
            Arg::new("INCLUDE_HIDDEN").help("Include hidden files").short('a')
                .long("show-hidden-files").action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("FOLLOW_LINKS").help("Follow symbolic links").short('l').long("follow-links")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("MAX_DEPTH").help("Limit the depth of subdirectories included").short('d')
                .long("depth").value_parser(RangedU64ValueParser::<usize>::new())
        )
        .next_help_heading("Patterns")
        .arg(
            Arg::new("STRING").help("A string to match within files").short('s').long("match-string")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("REGEX").help("A regex to match within files").short('r').long("match-regex")
                .action(ArgAction::Append)
        )
        .next_help_heading("Output")
        .arg(
            Arg::new("OUTPUT_FILE").help("TODO").short('o').long("output-file")
                .value_parser(ValueParser::path_buf()).value_hint(ValueHint::FilePath)
        )
        .arg(
            Arg::new("OUTPUT_FORMAT").help("TODO").short('f').long("output-format")
                .default_value(
                    "%bold%%file_once%%line_once%\
                    %clear%%blue%@@ %x%,%y% @@\n\
                    %white%%context_before%\n\
                    %green%%before%%bold%%match%%clear%%after%\n\
                    %white%%context_after%\n"
                ).value_parser(ValueParser::string())
        )
        .arg(
            Arg::new("CONTEXT").help("TODO").short('c').long("context-lines").default_value("3")
                .value_parser(RangedU64ValueParser::<usize>::new())
        )
        .get_matches();

    let mut patterns: Vec<Regex> = Vec::new();

    if let Some(values) = matches.get_many::<String>("STRING") {
        patterns.append(&mut values.cloned().filter_map(|value: String| {
            let escaped_value: &str = &fancy_regex::escape(&value);
            return Regex::new(escaped_value).ok();
        }).collect::<Vec<Regex>>());
        // .collect_into::<Vec<Regex>>(&mut patterns); Is currently experimental
    }

    if let Some(values) = matches.get_many::<String>("REGEX") {
        patterns.append(&mut values.cloned().filter_map(|value: String| {
            return Regex::new(&value).ok();
        }).collect::<Vec<Regex>>());
    }

    let mut output_file: Option<PathBuf> = matches.get_one::<PathBuf>("OUTPUT_FILE").cloned();
    if let Some(e) = output_file {
        output_file = e.canonicalize().ok();
    }

    return Args {
        paths: matches.get_many::<PathBuf>("PATH").expect("PATH is required").cloned()
            .collect::<Vec<PathBuf>>(),
        patterns,
        output_file,
        output_format: unescape(
            &matches.get_one::<String>("OUTPUT_FORMAT").cloned()
                .expect("OUTPUT_FORMAT has a default value")
        ).expect("unescape shouldn't error"), //TODO: can't input "\\"
        context: matches.get_one::<usize>("CONTEXT").copied()
            .expect("CONTEXT has a default value"),
        include_hidden: matches.get_flag("INCLUDE_HIDDEN"),
        follow_links: matches.get_flag("FOLLOW_LINKS"),
        max_depth: matches.get_one::<usize>("MAX_DEPTH").copied(),
    };
}

fn get_all_files(args: &Args) -> HashSet<PathBuf> {
    let mut files: HashSet<PathBuf> = HashSet::new();

    let mut insert_file = |file: Result<DirEntry, walkdir::Error>| {
        // Chaining this is kinda messy, but ignores any errors
        if let Ok(file_entry) = file {
            if file_entry.path().is_file() {
                if let Ok(abs_path) = file_entry.path().canonicalize() {
                    files.insert(abs_path);
                } //TODO: use match and throw an error
            }
        } //TODO: use match and throw an error
    };

    for path in &args.paths {
        let mut dir_walker: WalkDir = WalkDir::new(path).follow_links(args.follow_links);
        if let Some(depth) = args.max_depth {
            dir_walker = dir_walker.max_depth(depth);
        }
        
        let walker: IntoIter = dir_walker.into_iter();
        if !args.include_hidden {
            for file in walker.filter_entry(|entry: &DirEntry|
                (
                    !entry.file_name()
                        .to_str()
                        .map(|s: &str| s.starts_with('.'))
                        .unwrap_or(false)
                ) || (
                    entry.depth() == 0
                )
            ) {
                insert_file(file);
            }
        } else {
            for file in walker {
                insert_file(file);
            }
        }
    }
    return files;
}

fn find_pattern_in_files<'a>(
    files: &'a HashSet<PathBuf>,
    patterns: &'a Vec<Regex>
) -> Vec<FileFoundPatterns<'a>> {
    return files.iter().filter_map(|file: &PathBuf| {
        if let Ok(contents) = fs::read_to_string(file) {
            let mut found_patterns: Vec<FoundPattern> = Vec::new();
            for pattern in patterns {
                let matches: Vec<Match> = find_all_matches(pattern, &contents);
                found_patterns.append(&mut matches.into_iter().map(|found: Match| FoundPattern {
                    pattern,
                    start: found.start(),
                    end: found.end(),
                    // matched: found.as_str().to_owned()
                }).collect::<Vec<FoundPattern>>());
            }
            if found_patterns.is_empty() { return None };
            return Some(FileFoundPatterns {
                file,
                contents,
                found_patterns,
            });
        } //TODO: use match and throw an error on read fail
        return None;
    }).collect::<Vec<FileFoundPatterns<'a>>>();
}

fn find_all_matches<'a>(pattern: &Regex, search: &'a str) -> Vec<Match<'a>> {
    let mut matches: Vec<Match> = Vec::new();
    let mut location: usize = 0;
    while let Ok(Some(found)) = pattern.find_from_pos(search, location) {
        matches.push(found);
        location = found.end();
    }
    return matches;
}

fn generate_output_for_file(file_found_patterns: &FileFoundPatterns, args: &Args) {
    let mut output_lines: Vec<ColoredString> = Vec::new();
    //TODO: can't handle matches on the first line kinda need to add 0 and the end to this array
    let mut is_first_pattern: bool = true;

    for found_pattern in &file_found_patterns.found_patterns {
        let newlines: Vec<usize> = file_found_patterns.contents.match_indices('\n')
            .map(|index| index.0).collect::<Vec<usize>>();
        let mut closest_start: usize = 0;
        while newlines[closest_start] < found_pattern.start {
            closest_start += 1;
        }
        closest_start -= 1;
        let mut closest_end: usize = closest_start;
        while newlines[closest_end] < found_pattern.end {
            closest_end += 1;
        }

        let newlines_first_index: usize =
            match (closest_start as isize - args.context as isize) >= 0 {
                true => newlines[closest_start - args.context] + 1,
                false => 0
            };
        let newlines_last_index: usize =
            match closest_end + args.context  < newlines.len() {
                true => newlines[closest_end + args.context],
                false => file_found_patterns.contents.as_str().len()
            };

        let contents: &str = file_found_patterns.contents.as_str();
        let mut context_before: String = contents
            [newlines_first_index..newlines[closest_start]].to_owned();
        let mut before: String = contents
            [newlines[closest_start] + 1..found_pattern.start].to_owned();
        let matched: String = contents
            [found_pattern.start..found_pattern.end].to_owned();
        let after: String = contents
            [found_pattern.end..newlines[closest_end]].to_owned();
        let mut context_after: String = contents
            [newlines[closest_end] + 1..newlines_last_index].to_owned();

        let mut relevant_lines: Vec<&str> = Vec::new();
        relevant_lines.extend(context_before.split('\n'));
        relevant_lines.push(&before);
        relevant_lines.extend(context_after.split('\n'));
        let mut least_spaces: Option<usize> = None;
        for line in relevant_lines {
            if line.is_empty() {
                continue;
            }
            let line_spaces: usize = line.chars().take_while(|ch| ch.is_whitespace()).count();
            match least_spaces {
                Some(some_least_spaces) => {
                    if line_spaces < some_least_spaces {
                        least_spaces = Some(line_spaces);
                    }
                },
                None => {
                    least_spaces = Some(line_spaces);
                },
            }
        }
        
        let x: String;
        match least_spaces {
            None | Some(0) | Some(1) => {
                x = 0.to_string();
            },
            Some(some_least_spaces) => {
                let to_remove: usize = some_least_spaces - 1;
                x = some_least_spaces.to_string();
                let unindent_line = |line: &str| -> String {
                    if line.is_empty() {
                        return line.to_owned();
                    }
                    return line[to_remove..].to_owned();
                };
                context_before = context_before.split('\n').map(unindent_line).collect::<Vec<String>>().join("\n");
                before = unindent_line(&before);
                context_after = context_after.split('\n').map(unindent_line).collect::<Vec<_>>().join("\n");
            },
        }

        let y: String = (closest_start + 2).to_string();

        output_lines.append(&mut resolve_output_values(
            args,
            file_found_patterns.file,
            OutputValues {
                context_before,
                before,
                matched,
                after,
                context_after,
                is_first_pattern,
                pattern: found_pattern.pattern,
                x,
                y,
            },
        ));
        is_first_pattern = false;
    }

    for line in output_lines {
        print!("{}", line);
    }
}

fn apply_styles(string: &str, styles: Styles, color: Color) -> ColoredString {
    return match styles {
        Styles::Clear => string.clear(),
        Styles::Bold => string.bold(),
        Styles::Italic => string.italic(),
        Styles::Underline => string.underline(),
        _ => ColoredString::from(string),
    }.color(color);
}

fn resolve_output_values(
    args: &Args,
    file: &Path,
    output_values: OutputValues,
) -> Vec<ColoredString> {
    let escaped_regex: Regex = Regex::new(r"%(\w+)%").unwrap();
    let mut output_lines: Vec<ColoredString> = Vec::new();
    let mut current_color: Color = Color::White;//
    let mut current_styles: Styles = Styles::Clear;

    let mut location: usize = 0;
    while let Ok(Some(escaped)) = escaped_regex.find_from_pos(&args.output_format, location) {
        // Resolve characters inbetween
        output_lines.push(
            apply_styles(
                &args.output_format[location..escaped.start()],
                current_styles,
                current_color
            )
        );
        // Resolve escaped
        let escaped_string: &str = &escaped.as_str()[1..escaped.as_str().len() - 1];
        if let Ok(color) = Color::from_str(escaped_string) {
            current_color = color;
        } else if let Some(styles) = match escaped_string {
            "clear" => Some(Styles::Clear),
            "bold" => Some(Styles::Bold),
            "italic" => Some(Styles::Italic),
            "underline" => Some(Styles::Underline),
            _ => None,
        } {
            current_styles = styles;
        } else {
            output_lines.push(
                apply_styles(
                    match escaped_string {
                        "file" => file.to_str().unwrap(),
                        "file_once" => if output_values.is_first_pattern
                            {file.to_str().unwrap()} else {""},
                        "line_once" => if output_values.is_first_pattern {"\n"} else {""},
                        "x" => output_values.x.as_str(),
                        "y" => output_values.y.as_str(),
                        "before" => output_values.before.as_str(),
                        "match" => output_values.matched.as_str(),
                        "pattern" => output_values.pattern.as_str(),
                        "after" => output_values.after.as_str(),
                        "context_before" => output_values.context_before.as_str(),
                        "context_after" => output_values.context_after.as_str(),
                        _ => escaped_string,
                    },
                    current_styles,
                    current_color
                )
            );
        }
        location = escaped.end();
    }
    // Resolve characters after last match
    output_lines.push(
        args.output_format[location..].to_owned().color(current_color)
    );
    return  output_lines;
}