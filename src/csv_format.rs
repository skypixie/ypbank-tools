use std::io::{BufRead, BufReader, Read, Write};

use crate::error::Error;
use crate::model::Transaction;

/// Read transactions
pub fn read<R: Read>(reader: R) -> Result<Vec<Transaction>, Error> {
    let mut lines = BufReader::new(reader).lines();

    let header = match lines.next() {
        Some(line) => line?,
        None => return Err(Error::InvalidHeader("файл пустой".to_string())),
    };

    if header.trim() != "date,category,kind,amount" {
        return Err(Error::InvalidHeader(header.trim().to_string()));
    }

    let mut transactions: Vec<Transaction> = Vec::new();
    for line in lines {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        let tx = Transaction::from_fields(line, &parts)?;
        transactions.push(tx);
    }
    Ok(transactions)
}

/// Write transactions
pub fn write<W: Write>(transactions: &[Transaction], mut writer: W) -> Result<(), Error> {
    writeln!(writer, "date,category,kind,amount")?;
    for tx in transactions {
        writeln!(
            writer,
            "{},{},{},{}",
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

    const SAMPLE: &str = "date,category,kind,amount\n2026-04-01,salary,income,120000\n2026-04-02,rent,expense,40000\n";

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
    fn reads_csv_with_header() {
        let txs = read(Cursor::new(SAMPLE)).unwrap();

        assert_eq!(txs, sample_transactions());
    }

    #[test]
    fn trims_spaces_around_fields() {
        let input = "date,category,kind,amount\n2026-04-01, salary , income , 120000 \n";

        let txs = read(Cursor::new(input)).unwrap();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].category, "salary");
        assert_eq!(txs[0].amount, 120000);
    }

    #[test]
    fn reads_header_only_as_empty_list() {
        let txs = read(Cursor::new("date,category,kind,amount\n")).unwrap();

        assert!(txs.is_empty());
    }

    #[test]
    fn rejects_empty_input() {
        let err = read(Cursor::new("")).unwrap_err();

        assert!(matches!(err, Error::InvalidHeader(_)));
    }

    #[test]
    fn rejects_bad_header() {
        let err = read(Cursor::new("date,cat,kind,amount\n")).unwrap_err();

        assert!(matches!(err, Error::InvalidHeader(_)));
    }

    #[test]
    fn rejects_wrong_field_count() {
        let err = read(Cursor::new(
            "date,category,kind,amount\n2026-04-01,salary,income\n",
        ))
        .unwrap_err();

        assert!(matches!(err, Error::InvalidLine(_)));
    }

    #[test]
    fn rejects_invalid_date() {
        let err = read(Cursor::new(
            "date,category,kind,amount\n2026-04-01T00:00,salary,income,1\n",
        ))
        .unwrap_err();

        assert!(matches!(err, Error::InvalidDateFormat(_)));
    }

    #[test]
    fn rejects_bad_amount() {
        let err = read(Cursor::new(
            "date,category,kind,amount\n2026-04-01,salary,income,-1\n",
        ))
        .unwrap_err();

        assert!(matches!(err, Error::BadNumber(amount) if amount == "-1"));
    }

    #[test]
    fn writes_header_and_rows() {
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
}
