use clap::{App, Arg, ArgGroup};
use std::fs::read_dir;
use std::path::PathBuf;

// Programmer defined constants
static PROGRAM_NAME: &'static str = "annotate";

// Derived constants
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct Args {
    pub literal_files: Vec<PathBuf>,
    pub regex_files: Vec<PathBuf>,
    pub input_files: Vec<PathBuf>,
    pub keep_orig: bool,
    pub marker: String,
}

pub fn parse_args() -> Args {
    let args = App::new(PROGRAM_NAME)
        .version(VERSION)
        .author("ThermalSpan")
        .about(
            "A tool for annotating one set of files with the lines or regexes from other files",
        )
        .arg(
            Arg::with_name("LITERAL")
                .help("File with newline seperated literals to annotate")
                .long("literal")
                .short("l")
                .value_name("file.txt")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("REGEX")
                .help(
                    "File with newline seperated regular expressions to annotate",
                )
                .long("regex")
                .short("r")
                .value_name("file.txt")
                .takes_value(true),
        )
        .group(
            ArgGroup::with_name("INPUT")
                .args(&["LITERAL", "REGEX"])
                .required(true),
        )
        .arg(
            Arg::with_name("FILE")
                .help("File to annotate")
                .long("file")
                .short("f")
                .value_name("file.txt")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("DIR")
                .help("Directory of files to annotate")
                .long("dir")
                .short("d")
                .value_name("directory")
                .takes_value(true),
        )
        .group(
            ArgGroup::with_name("OUTPUT")
                .args(&["FILE", "DIR"])
                .required(true),
        )
        .arg(
            Arg::with_name("KEEP_ORIG")
                .help("Defualt behaivor is to delete original file")
                .long("keep-orig"),
        )
        .arg(
            Arg::with_name("MARKER")
                .help("The prefix for annotating, defaults to '+=1=+'")
                .short("m")
                .long("marker")
                .default_value("+=1=+")
                .takes_value(true),
        )
        .get_matches();

    let mut error_count = 0;
    let mut literal_files = Vec::new();
    let mut regex_files = Vec::new();
    let mut input_files = Vec::new();

    if let Some(raw_literal_paths) = args.values_of("LITERAL") {
        for raw_path in raw_literal_paths {
            handle_path(&raw_path, &mut literal_files, &mut error_count);
        }
    }

    if let Some(raw_regex_paths) = args.values_of("REGEX") {
        for raw_path in raw_regex_paths {
            handle_path(&raw_path, &mut regex_files, &mut error_count);
        }
    }

    if let Some(raw_input_paths) = args.values_of("FILE") {
        for raw_path in raw_input_paths {
            handle_path(&raw_path, &mut input_files, &mut error_count);
        }
    }

    if let Some(raw_dir_paths) = args.values_of("DIR") {
        for raw_path in raw_dir_paths {
            let path = PathBuf::from(raw_path);

            if !path.is_dir() {
                println!("ERROR: {} is not a directory", path.display());
                error_count += 1;
            }

            let contents: Vec<PathBuf> = read_dir(&path)
                .unwrap()
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap().path())
                .filter(|d| d.is_file())
                .collect();

            if contents.is_empty() {
                println!("WARN: {} did not contain any viable paths", path.display());
            }

            input_files.extend_from_slice(&contents);
        }
    }

    if input_files.is_empty() {
        println!("ERROR: no input files found");
        error_count += 1;
    }

    if error_count > 0 {
        println!("There were {} errors", error_count);
    }

    Args {
        literal_files: literal_files,
        regex_files: regex_files,
        input_files: input_files,
        keep_orig: args.is_present("KEEP_ORIG"),
        marker: String::from(args.value_of("MARKER").unwrap()),
    }
}

fn handle_path(
    raw_path: &str,
    collection: &mut Vec<PathBuf>,
    error_count: &mut usize,
) {
    let path = PathBuf::from(raw_path);
    if path.exists() {
        collection.push(path);
    } else {
        println!("ERROR: {} does not exist", path.display());
        *error_count += 1;
    }
}
