use std::io::{BufRead, BufReader, Read, Write};

use crate::error::Error;
use crate::model::Transaction;

/// Read transactions
pub fn read<R: Read>(reader: R) -> Result<Vec<Transaction>, Error> {
    let reader = BufReader::new(reader);
    let mut transactions: Vec<Transaction> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(';').map(|s| s.trim()).collect();

        let tx = Transaction::from_fields(line, &parts)?;
        transactions.push(tx);
    }
    Ok(transactions)
}

/// Write transactions
pub fn write<W: Write>(transactions: &[Transaction], mut writer: W) -> Result<(), Error> {
    for tx in transactions {
        writeln!(
            writer,
            "{};{};{};{}",
            tx.date,
            tx.category,
            tx.kind.as_string(),
            tx.amount
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Kind;
    use std::io::Cursor;

    const SAMPLE: &str = "2026-04-01;salary;income;120000\n2026-04-02;rent;expense;40000\n";

    fn sample_transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                date: "2026-04-01".to_string(),
                category: "salary".to_string(),
                kind: Kind::Income,
                amount: 120000,
            },
            Transaction {
                date: "2026-04-02".to_string(),
                category: "rent".to_string(),
                kind: Kind::Expense,
                amount: 40000,
            },
        ]
    }

    #[test]
    fn reads_all_transactions() {
        let txs = read(Cursor::new(SAMPLE)).unwrap();

        assert_eq!(txs, sample_transactions());
    }

    #[test]
    fn skips_blank_lines() {
        let input = "\n2026-04-01;salary;income;120000\n\n";

        let txs = read(Cursor::new(input)).unwrap();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].category, "salary");
    }

    #[test]
    fn reads_empty_input_as_empty_list() {
        let txs = read(Cursor::new("")).unwrap();

        assert!(txs.is_empty());
    }

    #[test]
    fn rejects_wrong_field_count() {
        let err = read(Cursor::new("2026-04-01;salary;income\n")).unwrap_err();

        assert!(matches!(err, Error::InvalidLine(_)));
    }

    #[test]
    fn rejects_invalid_date() {
        let err = read(Cursor::new("2026-13-01;salary;income;120000\n")).unwrap_err();

        assert!(matches!(err, Error::InvalidDateFormat(_)));
    }

    #[test]
    fn rejects_unknown_kind() {
        let err = read(Cursor::new("2026-04-01;salary;bonus;120000\n")).unwrap_err();

        assert!(matches!(err, Error::UnknownKind(kind) if kind == "bonus"));
    }

    #[test]
    fn rejects_bad_amount() {
        let err = read(Cursor::new("2026-04-01;salary;income;12x000\n")).unwrap_err();

        assert!(matches!(err, Error::BadNumber(amount) if amount == "12x000"));
    }

    #[test]
    fn writes_transactions() {
        let mut out = Vec::new();

        write(&sample_transactions(), &mut out).unwrap();

        assert_eq!(String::from_utf8(out).unwrap(), SAMPLE);
    }

    #[test]
    fn roundtrip_preserves_transactions() {
        let txs = read(Cursor::new(SAMPLE)).unwrap();
        let mut out = Vec::new();
        write(&txs, &mut out).unwrap();

        let reparsed = read(Cursor::new(out)).unwrap();

        assert_eq!(reparsed, txs);
    }

    #[test]
    fn propagates_read_errors() {
        struct FailingReader;

        impl std::io::Read for FailingReader {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("simulated failure"))
            }
        }

        let err = read(FailingReader).unwrap_err();

        assert!(matches!(err, Error::IoError(_)));
    }
}
