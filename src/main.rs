use clap::{Arg, Command, ValueHint, builder::ArgAction, builder::ValueParser, builder::RangedU64ValueParser};

use std::path::PathBuf;

use std::collections::HashSet;

#[derive(Debug)]
struct Args {
    paths: Vec<PathBuf>,
    output_file: Option<PathBuf>,
    output_format: String,
    include_hidden: bool,
    max_depth: u32,
}

fn main() {
    let args: Args = get_args();
    println!("{:?}", args);
    //verification?
    let files: HashSet<PathBuf> = get_all_files(args);
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
        output_file: matches.get_one::<PathBuf>("OUTPUT_FILE").cloned(),
        output_format: matches.get_one::<String>("OUTPUT_FORMAT").cloned().unwrap(),
        include_hidden: matches.get_flag("INCLUDE_HIDDEN"),
        max_depth: matches.get_one::<u32>("MAX_DEPTH").copied().unwrap(),
    };
}

fn get_all_files(args: Args) -> HashSet<PathBuf> {
    let files: HashSet<PathBuf> = HashSet::new();
    for path in args.paths {
        if path.is_file() {
            println!("Is file: {:?}", path);
        } else if path.is_dir() {
            println!("Is dir: {:?}", path);
        } else {
            println!("Is none: {:?}", path);
        }
    }
    return files;
}

// Input handling
// Input verification & interpretation
// Input execution
// Output