use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use crate::domain::Account;

const PRECISION: f64 = 10_000_000_000.0; // 10^10

#[derive(Clone, Copy)]
pub struct GBP(i128);

impl GBP {
    pub fn new(gbp: f64) -> Self {
        Self((gbp * PRECISION) as i128)
    }
}

impl From<&Account> for GBP {
    fn from(account: &Account) -> Self {
        Self(account.debit_balance())
    }
}

impl fmt::Display for GBP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "£{:.2}", self.0 as f64 / PRECISION)
    }
}

impl Add for GBP {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for GBP {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<i128> for GBP {
    type Output = Self;

    fn mul(self, rhs: i128) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<i128> for GBP {
    type Output = Self;

    fn div(self, rhs: i128) -> Self {
        Self(self.0 / rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scales_by_10_pow_10() {
        assert_eq!(GBP::new(1.0).0, 10_000_000_000);
        assert_eq!(GBP::new(-1.0).0, -10_000_000_000);
        assert_eq!(GBP::new(1.5).0, 15_000_000_000);
    }

    #[test]
    fn new_truncates_precision_finer_than_10_pow_minus_10_towards_zero() {
        // Anything past the 10th decimal digit is truncated, not rounded,
        // when converting from f64.
        assert_eq!(GBP::new(0.00000000019).0, 1);
        assert_eq!(GBP::new(-0.00000000019).0, -1);
    }

    #[test]
    fn new_handles_zero() {
        assert_eq!(GBP::new(0.0).0, 0);
    }

    #[test]
    fn display_formats_two_decimal_places() {
        assert_eq!(GBP::new(1.23).to_string(), "£1.23");
        assert_eq!(GBP::new(0.0).to_string(), "£0.00");
    }

    #[test]
    fn display_rounds_amounts_finer_than_two_decimal_places() {
        // Display only shows 2dp, so sub-penny precision that `new` stored
        // gets rounded away for presentation (but is still held internally).
        assert_eq!(GBP::new(1.006).to_string(), "£1.01");
        assert_eq!(GBP::new(1.004).to_string(), "£1.00");
    }

    #[test]
    fn display_formats_negative_values() {
        // The sign lands after the £ symbol, since we format a negative f64.
        assert_eq!(GBP::new(-1.2).to_string(), "£-1.20");
    }

    #[test]
    fn add_handles_negative_operands() {
        assert_eq!((GBP::new(1.0) + GBP::new(-1.5)).0, GBP::new(-0.5).0);
    }

    #[test]
    fn sub_can_go_negative() {
        assert_eq!((GBP::new(1.0) - GBP::new(2.0)).0, GBP::new(-1.0).0);
    }

    #[test]
    #[should_panic]
    fn add_overflow_panics() {
        let _ = GBP(i128::MAX) + GBP(1);
    }

    #[test]
    #[should_panic]
    fn sub_underflow_panics() {
        let _ = GBP(i128::MIN) - GBP(1);
    }

    #[test]
    #[should_panic]
    fn mul_overflow_panics() {
        let _ = GBP(i128::MAX) * 2;
    }

    #[test]
    fn mul_by_zero_yields_zero() {
        assert_eq!((GBP::new(42.0) * 0).0, 0);
    }

    #[test]
    fn mul_by_negative_flips_sign() {
        assert_eq!((GBP::new(1.5) * -1).0, GBP::new(-1.5).0);
    }

    #[test]
    fn div_truncates_towards_zero() {
        // 10 pence-tenths / 3 = 3.33.., truncated to 3, not rounded to 4.
        assert_eq!((GBP(10) / 3).0, 3);
        assert_eq!((GBP(-10) / 3).0, -3);
    }

    #[test]
    #[should_panic]
    fn div_by_zero_panics() {
        let _ = GBP::new(1.0) / 0;
    }

    #[test]
    #[should_panic]
    fn div_min_by_negative_one_overflows() {
        // i128::MIN / -1 overflows because i128::MAX < |i128::MIN|.
        let _ = GBP(i128::MIN) / -1;
    }
}
