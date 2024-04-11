use clap::{builder::{ArgAction, RangedU64ValueParser, ValueParser}, Arg, Command, ValueHint};
use fancy_regex::Regex;
use std::path::PathBuf;
use std::collections::HashSet;
use walkdir::{DirEntry, IntoIter, WalkDir};

#[derive(Debug)]
struct Args {
    paths: Vec<PathBuf>,
    _patterns: Vec<Regex>,
    _output_file: Option<PathBuf>,
    _output_format: String,
    include_hidden: bool,
    follow_links: bool,
    max_depth: Option<usize>,
}

fn main() {
    let args: Args = get_args();
    println!("{:?}", args);

    //TODO: verification?

    let files: HashSet<PathBuf> = get_all_files(&args);
    println!("{:?}", files);
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
    }

    if let Some(values) = matches.get_many::<String>("REGEX") {
        patterns.append(&mut values.cloned().filter_map(|value| {
            return Regex::new(&value).ok();
        }).collect::<Vec<Regex>>());
    }

    return Args {
        // Unwrap because required value:
        paths: matches.get_many::<PathBuf>("PATH").unwrap().cloned().collect::<Vec<PathBuf>>(),
        _patterns: patterns,
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
                }
            }
        }
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