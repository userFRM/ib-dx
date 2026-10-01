//! The decimal quantity the TWS API states sizes, positions and volumes in.

use std::fmt;
use std::str::FromStr;

/// A size, position or volume as the TWS API states it: a decimal number, or
/// nothing stated ([`UNSET_DECIMAL`]).
///
/// The value is held as a binary double, which keeps 15 significant decimal
/// digits exactly: every quantity the venue states fits, and it prints back as
/// the venue wrote it. The reference client's decimal keeps 16.
// ponytail: f64 inside; move to an exact decimal if a venue quantity ever needs a 16th digit.
#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Decimal(f64);

/// The decimal that states nothing, as the reference client's `UNSET_DECIMAL`.
pub const UNSET_DECIMAL: Decimal = Decimal(f64::MAX);

impl Decimal {
    /// Whether nothing was stated.
    pub fn is_unset(self) -> bool {
        self.0 == f64::MAX
    }

    /// The value as a double, or `None` when nothing was stated.
    pub fn to_f64(self) -> Option<f64> {
        (!self.is_unset()).then_some(self.0)
    }
}

impl From<f64> for Decimal {
    /// `f64::MAX`, this crate's unset double, becomes [`UNSET_DECIMAL`].
    fn from(value: f64) -> Self {
        Decimal(value)
    }
}

impl From<Decimal> for f64 {
    /// The double this crate uses elsewhere: `f64::MAX` when nothing was stated.
    fn from(value: Decimal) -> Self {
        value.0
    }
}

impl From<i64> for Decimal {
    fn from(value: i64) -> Self {
        Decimal(value as f64)
    }
}

impl FromStr for Decimal {
    type Err = std::num::ParseFloatError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim().parse().map(Decimal)
    }
}

impl fmt::Display for Decimal {
    /// The shortest text that reads back as this value; nothing when unset.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unset() { Ok(()) } else { fmt::Display::fmt(&self.0, f) }
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unset() { f.write_str("UNSET_DECIMAL") } else { fmt::Display::fmt(&self.0, f) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quantity_reads_back_as_the_venue_wrote_it_and_unset_states_nothing() {
        for text in ["0.00363473", "100", "1.5", "123456789.123456", "0.00000001"] {
            let d: Decimal = text.parse().unwrap();
            assert_eq!(d.to_string(), text);
        }
        assert!(Decimal::from(f64::MAX).is_unset());
        assert_eq!(UNSET_DECIMAL.to_f64(), None);
        assert_eq!(UNSET_DECIMAL.to_string(), "");
    }
}
