use std::fmt;

use crate::error::Error;

/// Transaction
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub date: String, // format: 2026-11-10
    pub category: String,
    pub kind: Kind,
    pub amount: u64,
}

impl Transaction {
    /// Create a transaction from line and fields
    pub fn from_fields(line: &str, fields: &[&str]) -> Result<Transaction, Error> {
        // Wrong number of fields in line
        if fields.len() != 4 {
            return Err(Error::InvalidLine(line.to_string()));
        }

        let date = fields[0];
        validate_date(date)?;

        let category = fields[1];
        let kind = Kind::parse(fields[2])?;
        let amount = fields[3]
            .parse::<u64>()
            .map_err(|_| Error::BadNumber(fields[3].to_string()))?;

        Ok(Transaction {
            date: date.to_string(),
            category: category.to_string(),
            kind,
            amount,
        })
    }
}

/// The first difference found between two transaction lists.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionDifference {
    Date {
        position: usize,
        left: String,
        right: String,
    },
    Category {
        position: usize,
        left: String,
        right: String,
    },
    Kind {
        position: usize,
        left: String,
        right: String,
    },
    Amount {
        position: usize,
        left: String,
        right: String,
    },
    ExtraInLeft {
        position: usize,
    },
    ExtraInRight {
        position: usize,
    },
}

impl TransactionDifference {
    /// Returns the 1-based position of the difference.
    pub fn position(&self) -> usize {
        match self {
            Self::Date { position, .. }
            | Self::Category { position, .. }
            | Self::Kind { position, .. }
            | Self::Amount { position, .. }
            | Self::ExtraInLeft { position }
            | Self::ExtraInRight { position } => *position,
        }
    }
}

impl fmt::Display for TransactionDifference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Date {
                position,
                left,
                right,
            } => write!(
                f,
                "transactions differ at position {position}: date: {left} != {right}"
            ),
            Self::Category {
                position,
                left,
                right,
            } => write!(
                f,
                "transactions differ at position {position}: category: {left} != {right}"
            ),
            Self::Kind {
                position,
                left,
                right,
            } => write!(
                f,
                "transactions differ at position {position}: kind: {left} != {right}"
            ),
            Self::Amount {
                position,
                left,
                right,
            } => write!(
                f,
                "transactions differ at position {position}: amount: {left} != {right}"
            ),
            Self::ExtraInLeft { position } => {
                write!(
                    f,
                    "extra transaction at position {position} in the left list"
                )
            }
            Self::ExtraInRight { position } => {
                write!(
                    f,
                    "extra transaction at position {position} in the right list"
                )
            }
        }
    }
}

// Transaction kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Income,
    Expense,
}

impl Kind {
    /// Turn kind to string
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }

    /// Parse kind from its text representation (`income` / `expense`).
    pub fn parse(s: &str) -> Result<Kind, Error> {
        match s {
            "income" => Ok(Kind::Income),
            "expense" => Ok(Kind::Expense),
            other => Err(Error::UnknownKind(other.to_string())),
        }
    }

    /// Turn byte to kind
    pub fn from_byte(byte: u8) -> Result<Self, Error> {
        match byte {
            0 => Ok(Self::Income),
            1 => Ok(Self::Expense),
            other => Err(Error::CorruptedBinaryFile(format!(
                "unknown kind byte: {other}"
            ))),
        }
    }

    /// Turn kind to byte
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Income => 0,
            Self::Expense => 1,
        }
    }
}

/// Validate date
pub fn validate_date(s: &str) -> Result<(), Error> {
    if s.len() != 10 {
        return Err(Error::InvalidDateFormat(s.to_string()));
    }

    let parts: Vec<&str> = s.split('-').collect();

    if !(parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && s.chars().filter(|&c| c != '-').all(|c| c.is_ascii_digit()))
    {
        return Err(Error::InvalidDateFormat(s.to_string()));
    }

    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| Error::InvalidDateFormat(s.to_string()))?;
    let day = parts[2]
        .parse::<u32>()
        .map_err(|_| Error::InvalidDateFormat(s.to_string()))?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(Error::InvalidDateFormat(s.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_dates() {
        for date in ["2026-04-01", "2024-02-29", "1999-12-31"] {
            assert!(validate_date(date).is_ok(), "{date} should be valid");
        }
    }

    #[test]
    fn rejects_invalid_dates() {
        for date in [
            "",
            "2026-4-01",
            "2026/04/01",
            "abcdefghij",
            "2026-00-10",
            "2026-13-01",
            "2026-04-00",
            "2026-04-32",
        ] {
            assert!(validate_date(date).is_err(), "{date} should be invalid");
        }
    }

    #[test]
    fn parses_kind_from_text() {
        assert_eq!(Kind::parse("income").unwrap(), Kind::Income);
        assert_eq!(Kind::parse("expense").unwrap(), Kind::Expense);
        assert!(matches!(Kind::parse("bonus"), Err(Error::UnknownKind(_))));
        assert!(matches!(Kind::parse("Income"), Err(Error::UnknownKind(_))));
    }

    #[test]
    fn formats_kind_as_lowercase() {
        assert_eq!(Kind::Income.as_string(), "income");
        assert_eq!(Kind::Expense.as_string(), "expense");
    }

    #[test]
    fn builds_transaction_from_fields() {
        let line = "2026-04-01;salary;income;120000";
        let fields = ["2026-04-01", "salary", "income", "120000"];

        let tx = Transaction::from_fields(line, &fields).unwrap();

        assert_eq!(
            tx,
            Transaction {
                date: "2026-04-01".to_string(),
                category: "salary".to_string(),
                kind: Kind::Income,
                amount: 120000,
            }
        );
    }

    #[test]
    fn rejects_transaction_with_wrong_field_count() {
        let line = "2026-04-01;salary;income";
        let fields = ["2026-04-01", "salary", "income"];

        assert!(matches!(
            Transaction::from_fields(line, &fields),
            Err(Error::InvalidLine(_))
        ));
    }
}
