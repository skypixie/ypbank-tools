use std::io::Cursor;

use ypbank_tools::format::Format;
use ypbank_tools::{read_transactions, write_transactions};

const CSV: &str = "date,category,kind,amount\n\
                   2026-04-01,salary,income,120000\n\
                   2026-04-02,rent,expense,40000\n";

#[test]
fn csv_to_bin_to_text() {
    let txs = read_transactions(Cursor::new(CSV), Format::Csv).unwrap();

    let mut bin = Vec::new();
    write_transactions(&txs, &mut bin, Format::Bin).unwrap();

    let from_bin = read_transactions(Cursor::new(bin), Format::Bin).unwrap();

    let mut text = Vec::new();
    write_transactions(&from_bin, &mut text, Format::Text).unwrap();

    assert_eq!(
        String::from_utf8(text).unwrap(),
        "2026-04-01;salary;income;120000\n2026-04-02;rent;expense;40000\n"
    );
}

#[test]
fn text_to_csv() {
    let txs = read_transactions(
        Cursor::new("2026-04-01;salary;income;120000\n"),
        Format::Text,
    )
    .unwrap();

    let mut csv = Vec::new();
    write_transactions(&txs, &mut csv, Format::Csv).unwrap();

    assert_eq!(
        String::from_utf8(csv).unwrap(),
        "date,category,kind,amount\n2026-04-01,salary,income,120000\n"
    );
}

#[test]
fn csv_to_bin_to_csv_keeps_data() {
    let txs = read_transactions(Cursor::new(CSV), Format::Csv).unwrap();

    let mut bin = Vec::new();
    write_transactions(&txs, &mut bin, Format::Bin).unwrap();

    let from_bin = read_transactions(Cursor::new(bin), Format::Bin).unwrap();

    let mut csv = Vec::new();
    write_transactions(&from_bin, &mut csv, Format::Csv).unwrap();

    assert_eq!(String::from_utf8(csv).unwrap(), CSV);
}

#[test]
fn empty_csv_converts_to_empty_list() {
    let txs = read_transactions(Cursor::new("date,category,kind,amount\n"), Format::Csv).unwrap();

    let mut bin = Vec::new();
    write_transactions(&txs, &mut bin, Format::Bin).unwrap();

    let from_bin = read_transactions(Cursor::new(bin), Format::Bin).unwrap();

    assert!(from_bin.is_empty());
}

#[test]
fn conversion_propagates_parse_errors() {
    assert!(read_transactions(Cursor::new("date,cat,kind,amount\n"), Format::Csv).is_err());
}
