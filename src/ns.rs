//! Bitmask flags controlling the natsort algorithm.
//!
//! These mirror [`natsort.ns`](https://natsort.readthedocs.io/en/stable/api.html#natsort.ns)
//! exactly — every hex value is measured from the real Python library so raw
//! integers can cross the `pyo3` bridge unchanged.
//!
//! See [DECISIONS.md §D-007](../DECISIONS.md#d-007) for why the planning docs
//! must not be trusted for flag values.

use bitflags::bitflags;

bitflags! {
    /// Bitmask flags that control the behaviour of the natsort algorithm.
    ///
    /// Values are measured from the real Python `natsort.ns` enum.
    /// Do **not** trust the hex values documented in `.agent/` — they are wrong.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NsFlags: u32 {
        /// Parse numbers as floats (`FLOAT = 0x1`).
        const FLOAT       = 0x01;
        /// Take sign (`+` / `-`) into account when matching numbers (`SIGNED = 0x2`).
        const SIGNED      = 0x02;
        /// Do not search for exponents as part of a float (`NOEXP = 0x4`).
        const NOEXP       = 0x04;
        /// Interpret strings as filesystem paths (`PATH = 0x8`).
        const PATH        = 0x08;
        /// Locale-aware sorting for alphabetical characters (`LOCALEALPHA = 0x10`).
        const LOCALEALPHA = 0x10;
        /// Locale-aware sorting for decimal/thousands separators (`LOCALENUM = 0x20`).
        const LOCALENUM   = 0x20;
        /// Ignore case when sorting (`IGNORECASE = 0x40`).
        const IGNORECASE  = 0x40;
        /// Lowercase letters sort before uppercase (`LOWERCASEFIRST = 0x80`).
        const LOWERCASEFIRST = 0x80;
        /// Group lowercase and uppercase letters together (`GROUPLETTERS = 0x100`).
        const GROUPLETTERS = 0x100;
        /// Capital-first grouping alias (`UNGROUPLETTERS = CAPITALFIRST = UG = 0x200`).
        const UNGROUPLETTERS = 0x200;
        /// Alias for `UNGROUPLETTERS`.
        const CAPITALFIRST = 0x200;
        /// Treat NaN / None as +Infinity so they sort last (`NANLAST = 0x400`).
        const NANLAST     = 0x400;
        /// Use NFKD unicode normalization instead of NFD (`COMPATIBILITYNORMALIZE = 0x800`).
        const COMPATIBILITYNORMALIZE = 0x800;
        /// Sort numbers after non-numbers (`NUMAFTER = 0x1000`).
        const NUMAFTER    = 0x1000;
        /// Presort input as strings to eliminate inconsistent ordering (`PRESORT = 0x2000`).
        const PRESORT     = 0x2000;

        // ------ Aliases --------------------------------------------

        /// Shortcut: `FLOAT | SIGNED` — useful for sorting real numbers.
        const REAL = Self::FLOAT.bits() | Self::SIGNED.bits();

        /// Shortcut: `LOCALEALPHA | LOCALENUM`.
        const LOCALE = Self::LOCALEALPHA.bits() | Self::LOCALENUM.bits();

        // ------ Default / no-op flags --------------------------------------------

        /// Default algorithm (equivalent to `INT`). Value is `0`.
        const DEFAULT = 0;
        /// Integer parsing mode (default). Value is `0`.
        const INT = 0;
        /// Unsigned number parsing (default). Value is `0`.
        const UNSIGNED = 0;

        // ------ Short aliases -------------------------------------------------

        /// Short alias for [`FLOAT`](NsFlags::FLOAT).
        const F = Self::FLOAT.bits();
        /// Short alias for [`SIGNED`](NsFlags::SIGNED).
        const S = Self::SIGNED.bits();
        /// Short alias for [`NOEXP`](NsFlags::NOEXP).
        const N = Self::NOEXP.bits();
        /// Short alias for [`PATH`](NsFlags::PATH).
        const P = Self::PATH.bits();
        /// Short alias for [`LOCALEALPHA`](NsFlags::LOCALEALPHA).
        const LA = Self::LOCALEALPHA.bits();
        /// Short alias for [`LOCALENUM`](NsFlags::LOCALENUM).
        const LN = Self::LOCALENUM.bits();
        /// Short alias for [`IGNORECASE`](NsFlags::IGNORECASE).
        const IC = Self::IGNORECASE.bits();
        /// Short alias for [`LOWERCASEFIRST`](NsFlags::LOWERCASEFIRST).
        const LF = Self::LOWERCASEFIRST.bits();
        /// Short alias for [`GROUPLETTERS`](NsFlags::GROUPLETTERS).
        const G = Self::GROUPLETTERS.bits();
        /// Short alias for [`UNGROUPLETTERS`](NsFlags::UNGROUPLETTERS).
        const UG = Self::UNGROUPLETTERS.bits();
        /// Short alias for [`NANLAST`](NsFlags::NANLAST).
        const NL = Self::NANLAST.bits();
        /// Short alias for [`COMPATIBILITYNORMALIZE`](NsFlags::COMPATIBILITYNORMALIZE).
        const CN = Self::COMPATIBILITYNORMALIZE.bits();
        /// Short alias for [`NUMAFTER`](NsFlags::NUMAFTER).
        const NA = Self::NUMAFTER.bits();
        /// Short alias for [`PRESORT`](NsFlags::PRESORT).
        const PS = Self::PRESORT.bits();
        /// Short alias for [`REAL`](NsFlags::REAL).
        const R = Self::REAL.bits();
        /// Short alias for [`LOCALE`](NsFlags::LOCALE).
        const L = Self::LOCALE.bits();
    }
}

impl Default for NsFlags {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_values_match_python() {
        assert_eq!(NsFlags::FLOAT.bits(), 0x1);
        assert_eq!(NsFlags::SIGNED.bits(), 0x2);
        assert_eq!(NsFlags::NOEXP.bits(), 0x4);
        assert_eq!(NsFlags::PATH.bits(), 0x8);
        assert_eq!(NsFlags::LOCALEALPHA.bits(), 0x10);
        assert_eq!(NsFlags::LOCALENUM.bits(), 0x20);
        assert_eq!(NsFlags::IGNORECASE.bits(), 0x40);
        assert_eq!(NsFlags::LOWERCASEFIRST.bits(), 0x80);
        assert_eq!(NsFlags::GROUPLETTERS.bits(), 0x100);
        assert_eq!(NsFlags::UNGROUPLETTERS.bits(), 0x200);
        assert_eq!(NsFlags::NANLAST.bits(), 0x400);
        assert_eq!(NsFlags::COMPATIBILITYNORMALIZE.bits(), 0x800);
        assert_eq!(NsFlags::NUMAFTER.bits(), 0x1000);
        assert_eq!(NsFlags::PRESORT.bits(), 0x2000);
        assert_eq!(NsFlags::REAL.bits(), 0x3);
        assert_eq!(NsFlags::LOCALE.bits(), 0x30);
        assert_eq!(NsFlags::DEFAULT.bits(), 0);
        assert_eq!(NsFlags::INT.bits(), 0);
        assert_eq!(NsFlags::UNSIGNED.bits(), 0);
    }

    #[test]
    fn short_aliases_match() {
        assert_eq!(NsFlags::F, NsFlags::FLOAT);
        assert_eq!(NsFlags::S, NsFlags::SIGNED);
        assert_eq!(NsFlags::N, NsFlags::NOEXP);
        assert_eq!(NsFlags::P, NsFlags::PATH);
        assert_eq!(NsFlags::LA, NsFlags::LOCALEALPHA);
        assert_eq!(NsFlags::LN, NsFlags::LOCALENUM);
        assert_eq!(NsFlags::IC, NsFlags::IGNORECASE);
        assert_eq!(NsFlags::R, NsFlags::REAL);
        assert_eq!(NsFlags::L, NsFlags::LOCALE);
    }

    #[test]
    fn combine_flags() {
        let combined = NsFlags::FLOAT | NsFlags::SIGNED;
        assert_eq!(combined, NsFlags::REAL);
    }
}
