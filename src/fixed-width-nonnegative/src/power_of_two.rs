// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::assertions_on_constants)]
//-
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_mut)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)] //? TODO: Remove temp development code
#![allow(non_snake_case)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

use crate::primitive_unsigned::{Number_Into_u64, PrimitiveUnsigned};

//=================================================================================================|

/// A number greater than or equal to `0`.
pub trait Nonnegative {}

impl<T> Nonnegative for T where T: PrimitiveUnsigned {}
impl Nonnegative for usize {}

/// A number greater than `0`.
pub trait Positive: Nonnegative {}
//impl Positive for Nonzero<PrimitiveUnsigned>
//impl Positive for Nonzero<usize>

/// A nonnegative even number.
pub trait NonnegativeEven: Nonnegative {}

/// A positive odd number.
pub trait PositiveOdd: Positive {}

/// A value that's 2^(0 <= n).
///
/// In LSF order, a (possibly empty) finite sequence of `0`-valued bits followed by a single `1`.
pub trait TwoRaisedToSomeNonnegativePower: Positive {
    /// The exponent to which `2` is raised.
    fn exponent(&self) -> u64;
}

/// A value that's 2^(0 <= n) - 1.
///
/// In LSF order, a (possibly empty) finite sequence of `1`-valued bits.
pub trait OneLessThanTwoRaisedToSomeNonnegativePower: Nonnegative {
    /// The exponent to which `2` is raised (before `1` is subtracted).
    fn exponent(&self) -> u64;
}

/// The value `0`.
pub trait IsZero: NonnegativeEven + OneLessThanTwoRaisedToSomeNonnegativePower {}

/// The value `1`.
pub trait IsOne: PositiveOdd + TwoRaisedToSomeNonnegativePower {}

//=================================================================================================|

/// A nonnegative power of 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TwoRaisedToANonnegativePower(u64);

impl TwoRaisedToANonnegativePower {
    /// Given a nonnegative exponent, constructs a `TwoRaisedToANonnegativePower`.
    pub fn new<E: Number_Into_u64>(exponent: E) -> Self {
        TwoRaisedToANonnegativePower(exponent.into_u64())
    }

    /// Given a nonnegative exponent, constructs a `TwoRaisedToANonnegativePower`.
    pub fn new_opt<E: num_traits::cast::ToPrimitive>(exponent: E) -> Option<Self> {
        exponent.to_u64().map(Self::new)
    }

    fn fmt_hex(&self, upper: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.write_str("0x")?;
        }
        let trailing_zero_bytes = self.0 / 8;
        let shift = (self.exponent() - trailing_zero_bytes * 8) as u8;
        let val = 1_u8 << shift;
        if upper {
            f.write_fmt(format_args!("{val:02X}"))?;
        } else {
            f.write_fmt(format_args!("{val:02x}"))?;
        }

        // Unwrap() is justified here because any sequence of bytes must fit into usize.
        #[allow(clippy::unwrap_used)]
        let trailing_zero_bytes: usize = trailing_zero_bytes.try_into().unwrap();

        f.write_str("00".repeat(trailing_zero_bytes).as_str())
    }
}

impl Nonnegative for TwoRaisedToANonnegativePower {}

impl Positive for TwoRaisedToANonnegativePower {}

impl OneLessThanTwoRaisedToSomeNonnegativePower for TwoRaisedToANonnegativePower {
    /// The exponent to which `2` is raised.
    fn exponent(&self) -> u64 {
        self.0
    }
}

/// To uppercase hex.
impl std::fmt::UpperHex for TwoRaisedToANonnegativePower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_hex(true, f)
    }
}

/// To lowercase hex.
impl std::fmt::LowerHex for TwoRaisedToANonnegativePower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_hex(false, f)
    }
}

//=================================================================================================|

/// A value which is 2^N - 1, where N is Nonnegative.
pub struct OneLessThanTwoRaisedToANonnegativePower(TwoRaisedToANonnegativePower);

impl OneLessThanTwoRaisedToANonnegativePower {
    /// Given a nonnegative exponent, constructs a `OneLessThanTwoRaisedToANonnegativePower`.
    pub fn new<E: Number_Into_u64>(exponent: E) -> Self {
        OneLessThanTwoRaisedToANonnegativePower(TwoRaisedToANonnegativePower::new(exponent))
    }

    /// Given a nonnegative exponent, constructs a `OneLessThanTwoRaisedToANonnegativePower`.
    pub fn new_opt<E: num_traits::cast::ToPrimitive>(exponent: E) -> Option<Self> {
        TwoRaisedToANonnegativePower::new_opt(exponent).map(OneLessThanTwoRaisedToANonnegativePower)
    }

    fn fmt_hex(&self, upper: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.write_str("0x")?;
        }
        let e: u64 = self.0.exponent();
        let mut full_bytes = self.0.exponent() / 8;
        let shift = (self.0.exponent() - full_bytes * 8) as u8;
        let val = (1_u8 << shift) - 1;
        if upper {
            f.write_fmt(format_args!("{val:02X}"))?;
        } else {
            f.write_fmt(format_args!("{val:02x}"))?;
        }
        for _ in 0..full_bytes {
            f.write_str("FF")?;
        }
        Ok(())
    }
}

impl Nonnegative for OneLessThanTwoRaisedToANonnegativePower {}

impl OneLessThanTwoRaisedToSomeNonnegativePower for OneLessThanTwoRaisedToANonnegativePower {
    /// The exponent to which `2` is raised (before `1` is subtracted).
    fn exponent(&self) -> u64 {
        self.0.exponent()
    }
}

/// To uppercase hex.
impl std::fmt::UpperHex for OneLessThanTwoRaisedToANonnegativePower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_hex(true, f)
    }
}

/// To lowercase hex.
impl std::fmt::LowerHex for OneLessThanTwoRaisedToANonnegativePower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_hex(false, f)
    }
}

//=================================================================================================|
