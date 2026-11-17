use std::env;
use std::fs::File;
use std::io::{self, Write, stdout};

use ypbank_tools::format::{Format, string_to_format};
use ypbank_tools::{read_transactions, write_transactions};

/// Run options
struct Options {
    input: Option<String>,
    input_format: Format,
    output_format: Format,
}

fn print_usage() {
    println!(
        "Usage: converter --input-format <csv|text|bin> \
         --output-format <csv|text|bin> [--input <file>]"
    );
    println!();
    println!("Reads transactions from FILE, or from stdin when --input is omitted.");
    println!("Writes the converted transactions to stdout.");
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = None;
    let mut input_format = None;
    let mut output_format = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => {
                let value = args.next().ok_or("--input requires a value")?;
                input = Some(value);
            }
            "--input-format" => {
                let value = args.next().ok_or("--input-format requires a value")?;
                input_format = Some(string_to_format(&value)?);
            }
            "--output-format" => {
                let value = args.next().ok_or("--output-format requires a value")?;
                output_format = Some(string_to_format(&value)?);
            }
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }

    let options = Options {
        input,
        input_format: input_format.ok_or("missing --input-format")?,
        output_format: output_format.ok_or("missing --output-format")?,
    };

    let transactions = match &options.input {
        Some(path) => {
            let file = File::open(path).map_err(|e| format!("cannot open {path}: {e}"))?;
            read_transactions(file, options.input_format)?
        }
        None => {
            let stdin = io::stdin();
            read_transactions(stdin.lock(), options.input_format)?
        }
    };

    let stdout = stdout();
    let mut out = stdout.lock();

    write_transactions(&transactions, &mut out, options.output_format)?;
    out.flush()?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("converter: {err}");
        std::process::exit(1);
    }
}
