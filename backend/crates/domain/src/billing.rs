//! Basic billing (Patient_Management_Spec §5.8, PM-18).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use vd_common::AppError;

use crate::text::opt;
use crate::visit::VisitType;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    #[default]
    Cash,
    Bkash,
    Nagad,
    Card,
}

impl PaymentMethod {
    pub const ALL: [Self; 4] = [Self::Cash, Self::Bkash, Self::Nagad, Self::Card];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Bkash => "bkash",
            Self::Nagad => "nagad",
            Self::Card => "card",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_str() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BillStatus {
    Paid,
    Due,
}

impl BillStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paid => "paid",
            Self::Due => "due",
        }
    }
}

/// What the receptionist enters. Amounts are in taka.
#[derive(Clone, Debug, Deserialize)]
pub struct BillInput {
    pub fee: f64,
    #[serde(default)]
    pub discount: f64,
    pub discount_reason: Option<String>,
    pub paid: f64,
    #[serde(default)]
    pub method: PaymentMethod,
}

/// Checked, rounded amounts ready to store.
#[derive(Clone, Debug, PartialEq)]
pub struct Amounts {
    pub fee: f64,
    pub discount: f64,
    pub discount_reason: Option<String>,
    pub paid: f64,
    pub method: PaymentMethod,
    pub status: BillStatus,
}

const MAX_AMOUNT: f64 = 1_000_000.0;

pub fn round2(x: f64) -> f64 {
    // `+ 0.0` turns -0.0 (the sum of nothing) into 0.0.
    (x * 100.0).round() / 100.0 + 0.0
}

impl BillInput {
    pub fn validate(self) -> Result<Amounts, AppError> {
        let bad = |m: &str| Err(AppError::Validation(m.into()));
        let [fee, discount, paid] = [self.fee, self.discount, self.paid].map(round2);
        if [fee, discount, paid]
            .iter()
            .any(|x| !x.is_finite() || *x < 0.0 || *x > MAX_AMOUNT)
        {
            return bad("amounts must be between 0 and 1000000");
        }
        if discount > fee {
            return bad("discount cannot be more than the fee");
        }
        let total = round2(fee - discount);
        if paid > total {
            return bad("paid cannot be more than the amount payable");
        }
        let discount_reason = opt(self.discount_reason, "discount reason", 150)?;
        if discount > 0.0 && discount_reason.is_none() {
            return bad("give a reason for the discount");
        }
        Ok(Amounts {
            fee,
            discount,
            discount_reason: if discount > 0.0 {
                discount_reason
            } else {
                None
            },
            paid,
            method: self.method,
            status: if paid >= total {
                BillStatus::Paid
            } else {
                BillStatus::Due
            },
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Bill {
    pub id: u64,
    pub visit_id: u64,
    pub receipt_no: String,
    pub fee: f64,
    pub discount: f64,
    pub discount_reason: Option<String>,
    pub total: f64,
    pub paid: f64,
    pub due: f64,
    pub method: PaymentMethod,
    pub status: BillStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One line of the daily collection report.
#[derive(Clone, Debug, Serialize)]
pub struct CollectionRow {
    pub bill_id: u64,
    pub visit_id: u64,
    pub visit_date: NaiveDate,
    pub receipt_no: String,
    pub patient_id: u64,
    pub patient_code: String,
    pub patient_name: String,
    pub visit_type: VisitType,
    pub fee: f64,
    pub discount: f64,
    pub total: f64,
    pub paid: f64,
    pub due: f64,
    pub method: PaymentMethod,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct CollectionTotals {
    pub count: usize,
    pub fee: f64,
    pub discount: f64,
    pub total: f64,
    pub paid: f64,
    pub due: f64,
    /// Amount collected per payment method, in `PaymentMethod::ALL` order.
    pub by_method: Vec<MethodTotal>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MethodTotal {
    pub method: PaymentMethod,
    pub paid: f64,
    pub count: usize,
}

pub fn totals(rows: &[CollectionRow]) -> CollectionTotals {
    let sum = |f: fn(&CollectionRow) -> f64| round2(rows.iter().map(f).sum());
    CollectionTotals {
        count: rows.len(),
        fee: sum(|r| r.fee),
        discount: sum(|r| r.discount),
        total: sum(|r| r.total),
        paid: sum(|r| r.paid),
        due: sum(|r| r.due),
        by_method: PaymentMethod::ALL
            .into_iter()
            .map(|method| {
                let mine = rows.iter().filter(|r| r.method == method && r.paid > 0.0);
                MethodTotal {
                    method,
                    paid: round2(mine.clone().map(|r| r.paid).sum()),
                    count: mine.count(),
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(fee: f64, discount: f64, paid: f64) -> BillInput {
        BillInput {
            fee,
            discount,
            discount_reason: Some("poor patient".into()),
            paid,
            method: PaymentMethod::Bkash,
        }
    }

    #[test]
    fn computes_status() {
        let a = input(800.0, 200.0, 600.0).validate().unwrap();
        assert_eq!(a.status, BillStatus::Paid);
        let a = input(800.0, 0.0, 300.0).validate().unwrap();
        assert_eq!(a.status, BillStatus::Due);
        assert_eq!(a.discount_reason, None);
        let a = input(0.0, 0.0, 0.0).validate().unwrap();
        assert_eq!(a.status, BillStatus::Paid);
    }

    #[test]
    fn rejects_bad_amounts() {
        assert!(input(500.0, 600.0, 0.0).validate().is_err());
        assert!(input(500.0, 100.0, 450.0).validate().is_err());
        assert!(input(-1.0, 0.0, 0.0).validate().is_err());
        assert!(input(f64::NAN, 0.0, 0.0).validate().is_err());
        let mut no_reason = input(500.0, 100.0, 0.0);
        no_reason.discount_reason = Some("  ".into());
        assert!(no_reason.validate().is_err());
    }

    #[test]
    fn sums_by_method() {
        let row = |paid: f64, due: f64, method| CollectionRow {
            bill_id: 1,
            visit_id: 1,
            visit_date: NaiveDate::from_ymd_opt(2026, 10, 12).unwrap(),
            receipt_no: "R-1".into(),
            patient_id: 1,
            patient_code: "P-1".into(),
            patient_name: "A".into(),
            visit_type: VisitType::New,
            fee: paid + due,
            discount: 0.0,
            total: paid + due,
            paid,
            due,
            method,
        };
        let t = totals(&[
            row(500.0, 0.0, PaymentMethod::Cash),
            row(300.5, 199.5, PaymentMethod::Bkash),
            row(800.0, 0.0, PaymentMethod::Cash),
            row(0.0, 500.0, PaymentMethod::Nagad),
        ]);
        assert_eq!(t.count, 4);
        assert_eq!(t.paid, 1600.5);
        assert_eq!(t.due, 699.5);
        assert_eq!(t.by_method[0].paid, 1300.0);
        assert_eq!(t.by_method[0].count, 2);
        assert_eq!(t.by_method[2].count, 0);
        assert!(t.by_method[2].paid.is_sign_positive());
    }
}
