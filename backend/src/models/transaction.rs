use chrono::{DateTime, Utc};

use super::validation::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Transfer,
    Deposit,
    Withdrawal,
    Payment,
}

impl TransactionType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transfer => "transfer",
            Self::Deposit => "deposit",
            Self::Withdrawal => "withdrawal",
            Self::Payment => "payment",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "transfer" => Some(Self::Transfer),
            "deposit" => Some(Self::Deposit),
            "withdrawal" => Some(Self::Withdrawal),
            "payment" => Some(Self::Payment),
            _ => None,
        }
    }
}

/// A transaction before PostgreSQL assigns its ID.
#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub external_reference: String,
    pub amount_minor: i64,
    pub currency: String,
    pub sender_account: String,
    pub recipient_account: String,
    pub transaction_type: TransactionType,
    pub occurred_at: DateTime<Utc>,
}

impl NewTransaction {
    /// Normalize fields that have a well-defined canonical form.
    pub fn normalize(mut self) -> Self {
        self.external_reference = self.external_reference.trim().to_owned();

        self.currency = self.currency.trim().to_ascii_uppercase();

        self.sender_account = self.sender_account.trim().to_owned();

        self.recipient_account = self.recipient_account.trim().to_owned();

        self
    }
}

/// A transaction persisted in PostgreSQL.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: i64,
    pub external_reference: String,
    pub amount_minor: i64,
    pub currency: String,
    pub sender_account: String,
    pub recipient_account: String,
    pub transaction_type: TransactionType,
    pub occurred_at: DateTime<Utc>,
}

impl Transaction {
    pub fn is_high_value(&self, threshold_minor: i64) -> bool {
        self.amount_minor >= threshold_minor
    }
}

fn validate_transaction_fields(
    external_reference: &str,
    amount_minor: i64,
    currency: &str,
    sender_account: &str,
    recipient_account: &str,
) -> Result<(), String> {
    if external_reference.trim().is_empty() {
        return Err(String::from("Transaction reference cannot be empty"));
    }

    if amount_minor <= 0 {
        return Err(String::from("Transaction amount must be greater than zero"));
    }

    if currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err(String::from(
            "Currency must be a three-letter uppercase code",
        ));
    }

    if sender_account.trim().is_empty() {
        return Err(String::from("Sender account cannot be empty"));
    }

    if recipient_account.trim().is_empty() {
        return Err(String::from("Recipient account cannot be empty"));
    }

    Ok(())
}

impl Validate for NewTransaction {
    fn validate(&self) -> Result<(), String> {
        validate_transaction_fields(
            &self.external_reference,
            self.amount_minor,
            &self.currency,
            &self.sender_account,
            &self.recipient_account,
        )
    }
}

impl Validate for Transaction {
    fn validate(&self) -> Result<(), String> {
        if self.id <= 0 {
            return Err(String::from("Transaction ID must be greater than zero"));
        }

        validate_transaction_fields(
            &self.external_reference,
            self.amount_minor,
            &self.currency,
            &self.sender_account,
            &self.recipient_account,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{NewTransaction, Transaction, TransactionType};
    use crate::models::validation::Validate;
    use chrono::Utc;

    fn sample_new_transaction() -> NewTransaction {
        NewTransaction {
            external_reference: String::from("BANK-TXN-001"),
            amount_minor: 125050,
            currency: String::from("INR"),
            sender_account: String::from("ACCOUNT-100"),
            recipient_account: String::from("ACCOUNT-200"),
            transaction_type: TransactionType::Transfer,
            occurred_at: Utc::now(),
        }
    }

    fn sample_transaction() -> Transaction {
        let new = sample_new_transaction();

        Transaction {
            id: 1,
            external_reference: new.external_reference,
            amount_minor: new.amount_minor,
            currency: new.currency,
            sender_account: new.sender_account,
            recipient_account: new.recipient_account,
            transaction_type: new.transaction_type,
            occurred_at: new.occurred_at,
        }
    }

    #[test]
    fn accepts_a_valid_new_transaction() {
        assert!(sample_new_transaction().validate().is_ok());
    }

    #[test]
    fn rejects_zero_or_negative_amounts() {
        let mut transaction = sample_new_transaction();
        transaction.amount_minor = 0;
        assert!(transaction.validate().is_err());

        transaction.amount_minor = -100;
        assert!(transaction.validate().is_err());
    }

    #[test]
    fn rejects_invalid_currency_codes() {
        let mut transaction = sample_new_transaction();

        transaction.currency = String::from("inr");
        assert!(transaction.validate().is_err());

        transaction.currency = String::from("US");
        assert!(transaction.validate().is_err());
    }

    #[test]
    fn rejects_empty_transaction_references() {
        let mut transaction = sample_new_transaction();
        transaction.external_reference = String::from("   ");

        assert!(transaction.validate().is_err());
    }

    #[test]
    fn rejects_missing_account_information() {
        let mut transaction = sample_new_transaction();
        transaction.sender_account.clear();
        assert!(transaction.validate().is_err());

        let mut transaction = sample_new_transaction();
        transaction.recipient_account.clear();
        assert!(transaction.validate().is_err());
    }

    #[test]
    fn identifies_transactions_at_or_above_a_threshold() {
        let transaction = sample_transaction();

        assert!(transaction.is_high_value(125050));
        assert!(transaction.is_high_value(100000));
        assert!(!transaction.is_high_value(200000));
    }

    #[test]
    fn converts_transaction_types_to_and_from_database_values() {
        for kind in [
            TransactionType::Transfer,
            TransactionType::Deposit,
            TransactionType::Withdrawal,
            TransactionType::Payment,
        ] {
            assert_eq!(TransactionType::parse(kind.as_str()), Some(kind));
        }

        assert_eq!(TransactionType::parse("unknown"), None);
    }
}

#[test]
fn normalizes_reference_currency_and_account_identifiers() {
    let mut transaction = sample_new_transaction();

    transaction.external_reference = String::from("  BANK-TXN-001  ");
    transaction.currency = String::from(" inr ");
    transaction.sender_account = String::from(" ACCOUNT-100 ");
    transaction.recipient_account = String::from(" ACCOUNT-200 ");

    let transaction = transaction.normalize();

    assert_eq!(transaction.external_reference, "BANK-TXN-001");
    assert_eq!(transaction.currency, "INR");
    assert_eq!(transaction.sender_account, "ACCOUNT-100");
    assert_eq!(transaction.recipient_account, "ACCOUNT-200");
    assert!(transaction.validate().is_ok());
}

#[test]
fn rejects_fields_that_are_empty_after_normalization() {
    let mut transaction = sample_new_transaction();
    transaction.external_reference = String::from("   ");

    let transaction = transaction.normalize();

    assert!(transaction.validate().is_err());
}

#[test]
fn parses_transaction_types_with_whitespace_and_mixed_case() {
    assert_eq!(
        TransactionType::parse(" TRANSFER "),
        Some(TransactionType::Transfer)
    );
    assert_eq!(
        TransactionType::parse("Payment"),
        Some(TransactionType::Payment)
    );
    assert_eq!(TransactionType::parse("unknown"), None);
}
