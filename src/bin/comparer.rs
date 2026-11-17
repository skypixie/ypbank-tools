use std::env;
use std::fs::File;

use ypbank_tools::format::{Format, string_to_format};
use ypbank_tools::model::Transaction;
use ypbank_tools::{compare, read_transactions};

/// Comparer options
struct Options {
    left: String,
    left_format: Format,
    right: String,
    right_format: Format,
}

fn print_usage() {
    println!(
        "Usage: comparer --left <file> --left-format <csv|text|bin> \
         --right <file> --right-format <csv|text|bin>"
    );
    println!();
    println!("Parses both files and prints the first transaction difference.");
}

fn read_file(path: &str, format: Format) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let file = File::open(path).map_err(|e| format!("cannot open {path}: {e}"))?;
    Ok(read_transactions(file, format)?)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut left = None;
    let mut left_format = None;
    let mut right = None;
    let mut right_format = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--left" => left = Some(args.next().ok_or("--left requires a value")?),
            "--left-format" => {
                let value = args.next().ok_or("--left-format requires a value")?;
                left_format = Some(string_to_format(&value)?);
            }
            "--right" => right = Some(args.next().ok_or("--right requires a value")?),
            "--right-format" => {
                let value = args.next().ok_or("--right-format requires a value")?;
                right_format = Some(string_to_format(&value)?);
            }
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }

    let options = Options {
        left: left.ok_or("missing --left")?,
        left_format: left_format.ok_or("missing --left-format")?,
        right: right.ok_or("missing --right")?,
        right_format: right_format.ok_or("missing --right-format")?,
    };

    let left = read_file(&options.left, options.left_format)?;
    let right = read_file(&options.right, options.right_format)?;

    match compare(&left, &right) {
        Some(diff) => println!("{diff}"),
        None => println!("files are equal"),
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("comparer: {err}");
        std::process::exit(1);
    }
}
