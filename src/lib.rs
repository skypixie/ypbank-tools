use std::io::{Read, Write};

pub mod bin_format;
pub mod csv_format;
pub mod error;
pub mod format;
pub mod model;
pub mod text_format;

use crate::error::Error;
use crate::format::Format;
use crate::model::{Transaction, TransactionDifference};

/// Reads a list of transactions from `reader` using the given `format`.
pub fn read_transactions<R: Read>(reader: R, format: Format) -> Result<Vec<Transaction>, Error> {
    match format {
        Format::Text => text_format::read(reader),
        Format::Csv => csv_format::read(reader),
        Format::Bin => bin_format::read(reader),
    }
}

/// Writes `transactions` to `writer` using the given `format`.
pub fn write_transactions<W: Write>(
    transactions: &[Transaction],
    writer: W,
    format: Format,
) -> Result<(), Error> {
    match format {
        Format::Text => text_format::write(transactions, writer),
        Format::Csv => csv_format::write(transactions, writer),
        Format::Bin => bin_format::write(transactions, writer),
    }
}

/// Compares two transaction lists element by element.
pub fn compare(left: &[Transaction], right: &[Transaction]) -> Option<TransactionDifference> {
    for (index, (left_tx, right_tx)) in left.iter().zip(right).enumerate() {
        if left_tx == right_tx {
            continue;
        }

        let position = index + 1;

        if left_tx.date != right_tx.date {
            return Some(TransactionDifference::Date {
                position,
                left: left_tx.date.clone(),
                right: right_tx.date.clone(),
            });
        }

        if left_tx.category != right_tx.category {
            return Some(TransactionDifference::Category {
                position,
                left: left_tx.category.clone(),
                right: right_tx.category.clone(),
            });
        }

        if left_tx.kind != right_tx.kind {
            return Some(TransactionDifference::Kind {
                position,
                left: left_tx.kind.as_string().to_string(),
                right: right_tx.kind.as_string().to_string(),
            });
        }

        if left_tx.amount != right_tx.amount {
            return Some(TransactionDifference::Amount {
                position,
                left: left_tx.amount.to_string(),
                right: right_tx.amount.to_string(),
            });
        }
    }

    if left.len() > right.len() {
        return Some(TransactionDifference::ExtraInLeft {
            position: right.len() + 1,
        });
    }

    if right.len() > left.len() {
        return Some(TransactionDifference::ExtraInRight {
            position: left.len() + 1,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Kind;

    fn tx(date: &str, category: &str, kind: Kind, amount: u64) -> Transaction {
        Transaction {
            date: date.to_string(),
            category: category.to_string(),
            kind,
            amount,
        }
    }

    fn sample() -> Vec<Transaction> {
        vec![
            tx("2026-04-01", "salary", Kind::Income, 120000),
            tx("2026-04-02", "rent", Kind::Expense, 40000),
        ]
    }

    #[test]
    fn equal_lists_have_no_difference() {
        assert_eq!(compare(&sample(), &sample()), None);
    }

    #[test]
    fn empty_lists_are_equal() {
        assert_eq!(compare(&[], &[]), None);
    }

    #[test]
    fn reports_date_difference() {
        let mut right = sample();
        right[0].date = "2026-04-09".to_string();

        assert_eq!(
            compare(&sample(), &right),
            Some(TransactionDifference::Date {
                position: 1,
                left: "2026-04-01".to_string(),
                right: "2026-04-09".to_string(),
            })
        );
    }

    #[test]
    fn reports_category_difference() {
        let mut right = sample();
        right[0].category = "bonus".to_string();

        assert_eq!(
            compare(&sample(), &right),
            Some(TransactionDifference::Category {
                position: 1,
                left: "salary".to_string(),
                right: "bonus".to_string(),
            })
        );
    }

    #[test]
    fn reports_kind_difference() {
        let mut right = sample();
        right[0].kind = Kind::Expense;

        assert_eq!(
            compare(&sample(), &right),
            Some(TransactionDifference::Kind {
                position: 1,
                left: "income".to_string(),
                right: "expense".to_string(),
            })
        );
    }

    #[test]
    fn reports_amount_difference_at_correct_position() {
        let mut right = sample();
        right[1].amount = 6500;

        let diff = compare(&sample(), &right).unwrap();

        assert_eq!(
            diff,
            TransactionDifference::Amount {
                position: 2,
                left: "40000".to_string(),
                right: "6500".to_string(),
            }
        );
        assert_eq!(diff.position(), 2);
    }

    #[test]
    fn reports_extra_transaction_in_left() {
        let left = sample();
        let right = left[..1].to_vec();

        assert_eq!(
            compare(&left, &right),
            Some(TransactionDifference::ExtraInLeft { position: 2 })
        );
    }

    #[test]
    fn reports_extra_transaction_in_right() {
        let left = sample()[..1].to_vec();
        let right = sample();

        assert_eq!(
            compare(&left, &right),
            Some(TransactionDifference::ExtraInRight { position: 2 })
        );
    }

    #[test]
    fn reports_extra_transaction_when_one_list_is_empty() {
        assert_eq!(
            compare(&[], &sample()),
            Some(TransactionDifference::ExtraInRight { position: 1 })
        );
        assert_eq!(
            compare(&sample(), &[]),
            Some(TransactionDifference::ExtraInLeft { position: 1 })
        );
    }

    #[test]
    fn reports_field_difference_before_length_difference() {
        let left = sample();
        let right = vec![tx("2026-04-09", "salary", Kind::Income, 120000)];

        assert_eq!(
            compare(&left, &right),
            Some(TransactionDifference::Date {
                position: 1,
                left: "2026-04-01".to_string(),
                right: "2026-04-09".to_string(),
            })
        );
    }

    #[test]
    fn displays_field_difference() {
        let diff = TransactionDifference::Amount {
            position: 3,
            left: "6500".to_string(),
            right: "7000".to_string(),
        };

        assert_eq!(
            diff.to_string(),
            "transactions differ at position 3: amount: 6500 != 7000"
        );
    }

    #[test]
    fn displays_extra_transaction() {
        let diff = TransactionDifference::ExtraInLeft { position: 4 };

        assert_eq!(
            diff.to_string(),
            "extra transaction at position 4 in the left list"
        );
    }
}
