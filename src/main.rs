#![allow(clippy::needless_return)]
#![allow(clippy::print_literal)]
#![allow(clippy::vec_init_then_push)]

use clap::{builder::{ArgAction, RangedU64ValueParser, ValueParser}, Arg, Command, ValueHint};
use fancy_regex::{Match, Regex};
use std::{fs, path::PathBuf, str::FromStr};
use std::collections::HashSet;
use walkdir::{DirEntry, IntoIter, WalkDir};
use colored::{Color, ColoredString, Colorize, Styles};
use unescape::unescape;

#[derive(Debug)]
struct Args {
    paths: Vec<PathBuf>,
    patterns: Vec<Regex>,
    output_file: Option<PathBuf>,
    output_format: String,
    _context: u32,
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
    file: &'a PathBuf,
    contents: String,
    found_patterns: Vec<FoundPattern<'a>>
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
            Arg::new("PATH").help("TODO").action(ArgAction::Append).required(true)
                .value_parser(ValueParser::path_buf()).value_hint(ValueHint::AnyPath)
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
            Arg::new("REGEX").help("A regex to match within files").short('r').long("match-regex").action(ArgAction::Append)
        )
        .next_help_heading("Output")
        .arg(
            Arg::new("OUTPUT_FILE").help("TODO").short('o').long("output-file")
                .value_parser(ValueParser::path_buf()).value_hint(ValueHint::FilePath)
        )
        .arg(
            Arg::new("OUTPUT_FORMAT").help("TODO").short('f').long("output-format")
                .default_value(
                    "%bold%%file_path%\n\
                    %clear%%blue%@@ %x%,%y% @@\n\
                    %white%%context_before%\n\
                    %green%%before%%bold%%match%%clear%%after%\n\
                    %white%%context_after%\n"
                ).value_parser(ValueParser::string())
        )
        .arg(
            Arg::new("CONTEXT").help("TODO").short('c').long("context-lines").default_value("3")
                .value_parser(RangedU64ValueParser::<u32>::new())
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
        ).expect("unescape shouldn't error"),
        _context: matches.get_one::<u32>("CONTEXT").copied()
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

fn find_all_matches<'b>(pattern: &Regex, search: &'b str) -> Vec<Match<'b>> {
    let mut matches: Vec<Match> = Vec::new();
    let mut location: usize = 0;
    while let Ok(Some(found)) = pattern.find_from_pos(search, location) {
        matches.push(found);
        location = found.end();
    }
    return matches;
}

fn generate_output_for_file(file_found_patterns: &FileFoundPatterns, args: &Args) {
    let escaped_regex: Regex = Regex::new(r"%(\w+)%").unwrap();
    let mut output_lines: Vec<ColoredString> = Vec::new();
    let mut current_color: Color = Color::White;//
    let mut current_styles: Styles = Styles::Clear;

    let apply_styles = |string: &str, styles: Styles, color: Color| {
        match styles {
            Styles::Clear => string.clear(),
            Styles::Bold => string.bold(),
            Styles::Italic => string.italic(),
            Styles::Underline => string.underline(),
            _ => ColoredString::from(string),
        }.color(color)
    };

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
                        "file_path" => file_found_patterns.file.to_str().unwrap(),
                        "x" => escaped_string,
                        "y" => escaped_string,
                        "before" => escaped_string,
                        "match" => escaped_string,
                        "pattern" => escaped_string,
                        "after" => escaped_string,
                        "context_before" => escaped_string,
                        "context_after" => escaped_string,
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
        args.output_format[location..].to_string().color(current_color)
    );

    // output_lines.push(ColoredString::from("\n"));
    for line in output_lines {
        print!("{}", line);
    }
}