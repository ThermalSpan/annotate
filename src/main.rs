extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate regex;

mod args_and_usage;

use regex::{RegexSetBuilder, escape};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

quick_main!(run);

fn run() -> Result<()> {
    let args = args_and_usage::parse_args();

    let mut expressions = Vec::new();

    for literal_file in args.literal_files {

        let file = File::open(&literal_file).chain_err(|| {
            format!("Can't open literal file: {}", literal_file.display())
        })?;

        let mut reader = BufReader::new(file);

        for line_maybe in reader.lines() {
            let mut line = line_maybe.chain_err(|| {
                format!("Unable to read line from: {}", literal_file.display())
            })?;

            chomp(&mut line);

            expressions.push(escape(&line));
        }
    }

    for regex_file in args.regex_files {
        let file = File::open(&regex_file).chain_err(|| {
            format!("Can't open regex file: {}", regex_file.display())
        })?;

        let mut reader = BufReader::new(file);

        for line_maybe in reader.lines() {
            let mut line = line_maybe.chain_err(|| {
                format!("Unable to read line from: {}", regex_file.display())
            })?;

            chomp(&mut line);

            expressions.push(line.clone());
        }
    }

    let regex = RegexSetBuilder::new(&expressions)
		.size_limit(4294967000)
		.build()?;

    for input_file in args.input_files {

        let mut orig_path = input_file.clone();
        if !orig_path.set_extension("orig") {
            bail!(ErrorKind::UnableToAddOrigExtension);
        }

        fs::rename(&input_file, &orig_path).chain_err(|| {
            format!(
                "Unable to rename\n{}\nto\n{}",
                input_file.display(),
                orig_path.display()
            )
        })?;

        // Now lets write out the original file
        let mut output_file = File::create(&input_file).chain_err(|| {
            format!("Can't create ouput file: {}", input_file.display())
        })?;

        let mut orig_file = File::open(&orig_path).chain_err(|| {
            format!("Can't open orig file: {}", orig_path.display())
        })?;

        let mut reader = BufReader::new(orig_file);
        let mut writer = BufWriter::new(output_file);

        for line_maybe in reader.lines() {
            let mut line = line_maybe.chain_err(|| {
                format!("Unable to read line from: {}", orig_path.display())
            })?;

            if regex.is_match(&line) {
                writer.write(&args.marker.as_bytes())?;
            }

            writer.write(&line.as_bytes())?;

            writer.write("\n".as_bytes())?;

            expressions.push(escape(&line));
        }


        if !args.keep_orig {
            fs::remove_file(&orig_path).chain_err(|| {
                format!("Unable to delete original {}", orig_path.display())
            })?;
        }
    }

    Ok(())
}

error_chain! {
    errors {
        UnableToAddOrigExtension {
            description("We were unable to add the .orig extension the ouput path")
        }
    }

    foreign_links {
        Regex(regex::Error);
        IO(std::io::Error);
    }
}

fn chomp(input: &mut String) {
    match input.pop() {
        None | Some('\n') => (),
        Some(c) => input.push(c),
    };
}
