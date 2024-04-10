use clap::{Arg, Command, ValueHint, builder::ArgAction, builder::ValueParser, builder::RangedU64ValueParser};
use core::panic;
use std::path::PathBuf;
use std::fs::{self, DirEntry};
use std::collections::HashSet;

#[derive(Debug)]
struct Args {
    paths: Vec<PathBuf>,
    _output_file: Option<PathBuf>,
    _output_format: String,
    _include_hidden: bool,
    _max_depth: u32,
}

fn main() {
    let args: Args = get_args();
    println!("{:?}", args);
    //verification?
    let files: HashSet<PathBuf> = get_all_files(&args.paths);
    println!("{:?}", files);
}

fn get_args() -> Args {
    let matches = Command::new("todo")
        .author("Inferno214221")
        .version("0.1.0")
        .about("A CLI program to find all instances of TODO notes within a file or directory")
        .disable_version_flag(true)
        .arg(
            Arg::new("VERSION")
                .help("TODO")
                .short('v')
                .long("version")
                .action(ArgAction::Version)
        )
        .arg(
            Arg::new("PATH")
                .help("TODO")
                .action(ArgAction::Append)
                .required(true)
                .value_parser(ValueParser::path_buf())
                .value_hint(ValueHint::AnyPath)
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
                .default_value("")
                .value_parser(ValueParser::string())
        )
        .arg(
            Arg::new("INCLUDE_HIDDEN")
                .help("TODO")
                .short('a')
                .long("hidden")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("MAX_DEPTH")
                .help("TODO")
                .short('d')
                .long("depth")
                .default_value("0")
                .value_parser(RangedU64ValueParser::<u32>::new())
        )
        .get_matches();

    let mut paths: Vec<PathBuf> = Vec::new();
    for path in matches.get_many::<PathBuf>("PATH").unwrap() {
        paths.push(path.clone());
    }

    // matches.get_one::<PathBuf>("PATH").cloned().unwrap()
    return Args {
        paths: paths,
        _output_file: matches.get_one::<PathBuf>("OUTPUT_FILE").cloned(),
        _output_format: matches.get_one::<String>("OUTPUT_FORMAT").cloned().unwrap(),
        _include_hidden: matches.get_flag("INCLUDE_HIDDEN"),
        _max_depth: matches.get_one::<u32>("MAX_DEPTH").copied().unwrap(),
    };
}

fn get_all_files(paths: &Vec<PathBuf>) -> HashSet<PathBuf> {
    let mut files: HashSet<PathBuf> = HashSet::new();
    for path in paths {
        let mut abs_path: PathBuf = path.canonicalize().unwrap();
        while abs_path.is_symlink() {
            abs_path = fs::read_link(abs_path).unwrap();
        }
        if abs_path.is_file() {
            println!("Is file: {:?}", abs_path);
            files.insert(abs_path.to_owned());
        } else if abs_path.is_dir() {
            println!("Is dir: {:?}", abs_path);
            let contents = fs::read_dir(abs_path).unwrap().map(|entry| {
                return entry.unwrap().path();
            }).collect::<Vec<PathBuf>>();
            println!("{:?}", contents);
            files.extend(get_all_files(&contents));
        } else {
            println!("Doesn't exist: {:?}", abs_path);
        }
    }
    return files;
}