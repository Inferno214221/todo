use clap::{builder::{ArgAction, RangedU64ValueParser, ValueParser}, Arg, Command, ValueHint};
use fancy_regex::{Match, Regex};
use std::{fs, path::PathBuf};
use std::collections::HashSet;
use walkdir::{DirEntry, IntoIter, WalkDir};

#[derive(Debug)]
struct Args {
    paths: Vec<PathBuf>,
    patterns: Vec<Regex>,
    _output_file: Option<PathBuf>,
    _output_format: String,
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
    found_patterns: Vec<FoundPattern<'a>>
}

fn main() {
    let args: Args = get_args();
    println!("{:?}", args);

    //TODO: verification?

    let files: HashSet<PathBuf> = get_all_files(&args);
    println!("{:?}", files);

    println!("{:?}", find_pattern_in_files(&files, &args.patterns));
}

fn get_args() -> Args {
    let matches = Command::new("todo")
        .author("Inferno214221")
        .version("0.1.0")
        .about("A CLI program to find all instances of TODO notes within a file or directory")
        .disable_version_flag(true)
        .arg(
            Arg::new("PATH")
                .help("TODO")
                .action(ArgAction::Append)
                .required(true)
                .value_parser(ValueParser::path_buf())
                .value_hint(ValueHint::AnyPath)
        )
        .arg(
            Arg::new("STRING")
                .help("TODO")
                .short('s')
                .long("string")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("REGEX")
                .help("TODO")
                .short('r')
                .long("regex")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("OUTPUT_FILE")
                .help("TODO")
                .short('o')
                .long("output")
                .value_parser(ValueParser::path_buf())
                .value_hint(ValueHint::FilePath)
        )
        .arg(
            Arg::new("OUTPUT_FORMAT")
                .help("TODO")
                .short('f')
                .long("format")
                .default_value("TODO")
                .value_parser(ValueParser::string())
        )
        .arg(
            Arg::new("INCLUDE_HIDDEN")
                .help("Include hidden files")
                .short('a')
                .long("hidden")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("FOLLOW_LINKS")
                .help("Follow symbolic links")
                .short('l')
                .long("follow")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("MAX_DEPTH")
                .help("Limit the depth of subdirectories included")
                .short('d')
                .long("depth")
                .value_parser(RangedU64ValueParser::<usize>::new())
        )
        .arg(
            Arg::new("VERSION")
                .help("Print version")
                .short('v')
                .long("version")
                .action(ArgAction::Version)
        )
        .get_matches();

    let mut patterns: Vec<Regex> = Vec::new();

    if let Some(values) = matches.get_many::<String>("STRING") {
        patterns.append(&mut values.cloned().filter_map(|value| {
            let escaped_value: &str = &fancy_regex::escape(&value);
            return Regex::new(escaped_value).ok();
        }).collect::<Vec<Regex>>());
        // .collect_into::<Vec<Regex>>(&mut patterns); Is currently experimental
    }

    if let Some(values) = matches.get_many::<String>("REGEX") {
        patterns.append(&mut values.cloned().filter_map(|value| {
            return Regex::new(&value).ok();
        }).collect::<Vec<Regex>>());
    }

    return Args {
        // Unwrap because required value:
        paths: matches.get_many::<PathBuf>("PATH").unwrap().cloned().collect::<Vec<PathBuf>>(),
        patterns,
        _output_file: matches.get_one::<PathBuf>("OUTPUT_FILE").cloned(),
        // Unwrap because default value:
        _output_format: matches.get_one::<String>("OUTPUT_FORMAT").cloned().unwrap(),
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
            for file in walker.filter_entry(|entry|
                (
                    !entry.file_name()
                        .to_str()
                        .map(|s| s.starts_with("."))
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

fn find_pattern_in_files<'a>(files: &'a HashSet<PathBuf>, patterns: &'a Vec<Regex>) -> Vec<FileFoundPatterns<'a>> {
    return files.into_iter().filter_map(|file| {
        if let Ok(contents) = fs::read_to_string(file) {
            let mut found_patterns: Vec<FoundPattern> = Vec::new();
            for pattern in patterns {
                let matches: Vec<Match> = find_all_matches(&pattern, &contents);
                found_patterns.append(&mut matches.into_iter().map(|found| FoundPattern {
                    pattern: &pattern,
                    start: found.start(),
                    end: found.end(),
                    // matched: found.as_str().to_owned()
                }).collect::<Vec<FoundPattern>>());
            }
            return Some(FileFoundPatterns {
                file: &file,
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