use std::io::{Read, Write};

use crate::error::Error;
use crate::model::{Kind, Transaction, validate_date};

const MAGIC: [u8; 3] = *b"YPB";
const VERSION: u8 = 1;
const MAX_STRING_LEN: u16 = 1024;

fn read_u8<R: Read>(reader: &mut R) -> Result<u8, Error> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16<R: Read>(reader: &mut R) -> Result<u16, Error> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64<R: Read>(reader: &mut R) -> Result<u64, Error> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_string<R: Read>(reader: &mut R) -> Result<String, Error> {
    let len = read_u16(reader)? as usize;

    if len > MAX_STRING_LEN as usize {
        return Err(Error::CorruptedBinaryFile(format!(
            "string too long: {len}"
        )));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;

    String::from_utf8(buf).map_err(|_| Error::CorruptedBinaryFile("invalid utf-8".to_string()))
}

fn write_string<W: Write>(writer: &mut W, s: &str) -> Result<(), Error> {
    if s.len() > MAX_STRING_LEN as usize {
        return Err(Error::StringTooLong(s.len()));
    }

    let len = s.len() as u16;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

/// Read bin format
pub fn read<R: Read>(mut reader: R) -> Result<Vec<Transaction>, Error> {
    // magic
    let mut magic = [0u8; 3];
    reader.read_exact(&mut magic)?;
    if magic != MAGIC {
        return Err(Error::CorruptedBinaryFile(format!(
            "bad magic: expected YPB, got {:?}",
            String::from_utf8_lossy(&magic)
        )));
    }

    // version
    let version = read_u8(&mut reader)?;
    if version != VERSION {
        return Err(Error::CorruptedBinaryFile(format!(
            "bad version: expected {VERSION}, got {version}"
        )));
    }

    // count
    let count = read_u32(&mut reader)?;

    // transactions
    let mut transactions = Vec::new();
    for _ in 0..count {
        let date = read_string(&mut reader)?;
        validate_date(&date)?;

        let category = read_string(&mut reader)?;

        let kind = Kind::from_byte(read_u8(&mut reader)?)?;

        let amount = read_u64(&mut reader)?;

        transactions.push(Transaction {
            date,
            category,
            kind,
            amount,
        });
    }

    Ok(transactions)
}

/// Write bin format
pub fn write<W: Write>(transactions: &[Transaction], mut writer: W) -> Result<(), Error> {
    writer.write_all(&MAGIC)?;
    writer.write_all(&[VERSION])?;
    writer.write_all(&(transactions.len() as u32).to_le_bytes())?;

    for tx in transactions {
        write_string(&mut writer, &tx.date)?;
        write_string(&mut writer, &tx.category)?;
        writer.write_all(&[tx.kind.to_byte()])?;
        writer.write_all(&tx.amount.to_le_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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

    fn encode(transactions: &[Transaction]) -> Vec<u8> {
        let mut out = Vec::new();
        write(transactions, &mut out).unwrap();
        out
    }

    fn push_string(out: &mut Vec<u8>, s: &str) {
        out.extend_from_slice(&(s.len() as u16).to_le_bytes());
        out.extend_from_slice(s.as_bytes());
    }

    /// Builds a valid file with a single record, letting tests corrupt one field.
    fn file_with_fields(date: &str, category: &str, kind: u8, amount: u64) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&MAGIC);
        out.push(VERSION);
        out.extend_from_slice(&1u32.to_le_bytes());
        push_string(&mut out, date);
        push_string(&mut out, category);
        out.push(kind);
        out.extend_from_slice(&amount.to_le_bytes());
        out
    }

    #[test]
    fn roundtrip_preserves_transactions() {
        let txs = sample_transactions();

        let decoded = read(Cursor::new(encode(&txs))).unwrap();

        assert_eq!(decoded, txs);
    }

    #[test]
    fn writes_magic_version_and_count() {
        let txs = sample_transactions();

        let encoded = encode(&txs);

        assert_eq!(&encoded[..3], &MAGIC[..]);
        assert_eq!(encoded[3], VERSION);
        assert_eq!(
            u32::from_le_bytes(encoded[4..8].try_into().unwrap()),
            txs.len() as u32
        );
    }

    #[test]
    fn roundtrips_empty_list() {
        let decoded = read(Cursor::new(encode(&[]))).unwrap();

        assert!(decoded.is_empty());
    }

    #[test]
    fn roundtrips_category_with_separators() {
        let txs = vec![Transaction {
            date: "2026-04-01".to_string(),
            category: "salary; bonus, extra".to_string(),
            kind: Kind::Income,
            amount: 1,
        }];

        let decoded = read(Cursor::new(encode(&txs))).unwrap();

        assert_eq!(decoded, txs);
    }

    #[test]
    fn rejects_bad_magic() {
        let err = read(Cursor::new(b"XYZ".to_vec())).unwrap_err();

        assert!(matches!(err, Error::CorruptedBinaryFile(_)));
    }

    #[test]
    fn rejects_unsupported_version() {
        let err = read(Cursor::new(b"YPB\x02".to_vec())).unwrap_err();

        assert!(matches!(err, Error::CorruptedBinaryFile(_)));
    }

    #[test]
    fn rejects_truncated_file() {
        let mut encoded = encode(&sample_transactions());
        encoded.pop();

        assert!(read(Cursor::new(encoded)).is_err());
    }

    #[test]
    fn rejects_unknown_kind_byte() {
        let file = file_with_fields("2026-04-01", "salary", 2, 120000);

        let err = read(Cursor::new(file)).unwrap_err();

        assert!(matches!(err, Error::CorruptedBinaryFile(_)));
    }

    #[test]
    fn rejects_invalid_date() {
        let file = file_with_fields("2026-13-01", "salary", 0, 120000);

        let err = read(Cursor::new(file)).unwrap_err();

        assert!(matches!(err, Error::InvalidDateFormat(_)));
    }

    #[test]
    fn rejects_invalid_utf8() {
        let mut file = Vec::new();
        file.extend_from_slice(&MAGIC);
        file.push(VERSION);
        file.extend_from_slice(&1u32.to_le_bytes());
        file.extend_from_slice(&2u16.to_le_bytes());
        file.extend_from_slice(&[0xFF, 0xFE]);

        let err = read(Cursor::new(file)).unwrap_err();

        assert!(matches!(err, Error::CorruptedBinaryFile(_)));
    }

    #[test]
    fn rejects_too_long_string_on_read() {
        let mut file = Vec::new();
        file.extend_from_slice(&MAGIC);
        file.push(VERSION);
        file.extend_from_slice(&1u32.to_le_bytes());
        file.extend_from_slice(&(MAX_STRING_LEN + 1).to_le_bytes());

        let err = read(Cursor::new(file)).unwrap_err();

        assert!(matches!(err, Error::CorruptedBinaryFile(_)));
    }

    #[test]
    fn rejects_too_long_string_on_write() {
        let too_long = "x".repeat(MAX_STRING_LEN as usize + 1);
        let txs = vec![Transaction {
            date: "2026-04-01".to_string(),
            category: too_long,
            kind: Kind::Income,
            amount: 1,
        }];

        let err = write(&txs, &mut Vec::new()).unwrap_err();

        assert!(matches!(err, Error::StringTooLong(len) if len == MAX_STRING_LEN as usize + 1));
    }
}
