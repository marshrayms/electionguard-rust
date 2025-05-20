// Copyright (C) Microsoft Corporation. All rights reserved.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(elided_lifetimes_in_paths)]
#![allow(clippy::map_flatten)] //? TODO: Remove temp development code
#![allow(clippy::needless_range_loop)] //? TODO: Remove temp development code

//=================================================================================================|

//! ## Traits useful for describing integer buffers.
//!
//! ### Four marker traits from the [zerocopy](https://crates.io/crates/zerocopy) crate:
//!
//! - [`zerocopy::FromZeroes`] "Indicates that a sequence of zero bytes represents a valid instance of a type"
//! - [`zerocopy::FromBytes`] "Indicates that a type may safely be converted from an arbitrary byte sequence"
//! - [`zerocopy::AsBytes`] "Indicates that a type may safely be converted *to* a byte sequence"
//!
//! ### Traits from the [bytemuck](https://crates.io/crates/bytemuck) crate:
//!
//! - [`AnyBitPattern`](`bytemuck::AnyBitPattern`) Types that are valid for any bit pattern.
//! - [`Pod`](`bytemuck::Pod`) "Plain old data" (subtrait of `AnyBitPattern`)
//! - [`Contiguous`](`bytemuck::Contiguous`) "Equivalent to some known integral type" and values
//!   fall within a contiguous fixed range.
//!   - Mainly for fieldless enums.
//! - [`NoUninit`](`bytemuck::NoUninit`) Like `Pod`, but not `Zeroable` or `AnyBitPattern`.
//! - [`CheckedBitPattern`](`bytemuck::CheckedBitPattern`) Allows checking to see if a bit pattern is valid.
//! - [`PodInOption`](`bytemuck::PodInOption`) Types that are Pod even when in an `Option`.
//!   E.g. `std::num::NonZeroU8`.
//! - [`Zeroable`](`bytemuck::Zeroable`) Types for which the all-zeroes bit pattern is valid.
//! - [`TransparentWrapper`](`bytemuck::TransparentWrapper`) `#[repr(transparent)]` types.
//! - [`ZeroableInOption`](`bytemuck::ZeroableInOption`)  Types for which all-zeroes bit pattern is
//!   valid for `Option<T>`.

pub const PRIMITIVEUNSIGNED_BITS_L2_MIN: u8 = 3;
pub const PRIMITIVEUNSIGNED_BITS_L2_MAX: u8 = 7;
pub const PRIMITIVEUNSIGNED_BITS_L2_RANGE: std::ops::Range<u8> =
    PRIMITIVEUNSIGNED_BITS_L2_MIN..(PRIMITIVEUNSIGNED_BITS_L2_MAX + 1);

pub const PRIMITIVEUNSIGNED_BITS_MIN: usize = 1 << PRIMITIVEUNSIGNED_BITS_L2_MIN;
pub const PRIMITIVEUNSIGNED_BITS_MAX: usize = 1 << PRIMITIVEUNSIGNED_BITS_L2_MAX;
pub const PRIMITIVEUNSIGNED_BITS_RANGE: std::ops::Range<usize> =
    PRIMITIVEUNSIGNED_BITS_MIN..(PRIMITIVEUNSIGNED_BITS_MAX + 1);

//? TODO Cf. `funty::Unsigned`

/// Trait to identify fixed size native primitive unsigned integer types of.
#[rustfmt::skip]
pub trait PrimitiveUnsigned:
    Sized
    + Clone
    + Copy
    + Default
    + std::fmt::Debug
    + std::fmt::Display
    + std::fmt::Binary
    + std::fmt::LowerHex
    + std::fmt::UpperHex
    + std::cmp::PartialEq<Self>
    + std::cmp::Eq
    + std::cmp::PartialOrd<Self>
    + std::cmp::Ord
    + std::ops::Shl<u8, Output = Self>
    + std::ops::Shl<u16, Output = Self>
    + std::ops::Shl<u32, Output = Self>
    + std::ops::Shl<u64, Output = Self>
    + std::ops::Shl<u128, Output = Self>
    + std::ops::Shl<usize, Output = Self>
    + std::ops::Shl<i8, Output = Self>
    + std::ops::Shl<i16, Output = Self>
    + std::ops::Shl<i32, Output = Self>
    + std::ops::Shl<i64, Output = Self>
    + std::ops::Shl<i128, Output = Self>
    + std::ops::Shl<isize, Output = Self>
    + std::ops::ShlAssign<u8>
    + std::ops::ShlAssign<u16>
    + std::ops::ShlAssign<u32>
    + std::ops::ShlAssign<u64>
    + std::ops::ShlAssign<u128>
    + std::ops::ShlAssign<usize>
    + std::ops::ShlAssign<i8>
    + std::ops::ShlAssign<i16>
    + std::ops::ShlAssign<i32>
    + std::ops::ShlAssign<i64>
    + std::ops::ShlAssign<i128>
    + std::ops::ShlAssign<isize>
    + std::ops::Shr<u8, Output = Self>
    + std::ops::Shr<u16, Output = Self>
    + std::ops::Shr<u32, Output = Self>
    + std::ops::Shr<u64, Output = Self>
    + std::ops::Shr<u128, Output = Self>
    + std::ops::Shr<usize, Output = Self>
    + std::ops::Shr<i8, Output = Self>
    + std::ops::Shr<i16, Output = Self>
    + std::ops::Shr<i32, Output = Self>
    + std::ops::Shr<i64, Output = Self>
    + std::ops::Shr<i128, Output = Self>
    + std::ops::Shr<isize, Output = Self>
    + std::ops::ShrAssign<u8>
    + std::ops::ShrAssign<u16>
    + std::ops::ShrAssign<u32>
    + std::ops::ShrAssign<u64>
    + std::ops::ShrAssign<u128>
    + std::ops::ShrAssign<usize>
    + std::ops::ShrAssign<i8>
    + std::ops::ShrAssign<i16>
    + std::ops::ShrAssign<i32>
    + std::ops::ShrAssign<i64>
    + std::ops::ShrAssign<i128>
    + std::ops::ShrAssign<isize>
    + std::ops::Add<Self, Output = Self>
    + for<'a> std::ops::Add<&'a Self, Output = Self>
    + std::ops::AddAssign<Self>
    + for<'a> std::ops::AddAssign<&'a Self>
    + std::ops::BitAnd<Self, Output = Self>
    + for<'a> std::ops::BitAnd<&'a Self, Output = Self>
    + std::ops::BitAndAssign<Self>
    + for<'a> std::ops::BitAndAssign<&'a Self>
    + std::ops::BitOr<Self, Output = Self>
    + for<'a> std::ops::BitOr<&'a Self, Output = Self>
    + std::ops::BitOrAssign<Self>
    + for<'a> std::ops::BitOrAssign<&'a Self>
    + std::ops::BitXor<Self, Output = Self>
    + for<'a> std::ops::BitXor<&'a Self, Output = Self>
    + std::ops::BitXorAssign<Self>
    + for<'a> std::ops::BitXorAssign<&'a Self>
    + std::ops::Div<Self, Output = Self>
    + for<'a> std::ops::Div<&'a Self, Output = Self>
    + std::ops::DivAssign<Self>
    + for<'a> std::ops::DivAssign<&'a Self>
    + std::ops::Mul<Self, Output = Self>
    + for<'a> std::ops::Mul<&'a Self, Output = Self>
    + std::ops::MulAssign<Self>
    + for<'a> std::ops::MulAssign<&'a Self>
    + std::ops::Rem<Self, Output = Self>
    + for<'a> std::ops::Rem<&'a Self, Output = Self>
    + std::ops::RemAssign<Self>
    + for<'a> std::ops::RemAssign<&'a Self>
    + std::ops::Sub<Self, Output = Self>
    + for<'a> std::ops::Sub<&'a Self, Output = Self>
    + std::ops::SubAssign<Self>
    + for<'a> std::ops::SubAssign<&'a Self>
    + std::convert::From<bool>
    + std::convert::From<u8>
    + std::convert::Into<u128>
    + bytemuck::Zeroable
    + bytemuck::Pod
    + zeroize::DefaultIsZeroes
{
    type Type;

    #[allow(non_camel_case_types)]
    type Type_array_u8: std::convert::AsRef<[u8]>;

    const NAME: &'static str;
    const BITS_L2: u8;
    const ZERO: Self;

    #[allow(non_upper_case_globals)]
    const ZERO_array_u8: Self::Type_array_u8;

    const ONE: Self;
    const MAX: Self;

    const ALIGN: usize = std::mem::align_of::<Self>();
    const SIZE: usize = std::mem::size_of::<Self>();
    const BITS: usize = 1 << Self::BITS_L2;
    const HEX_DIGITS: usize = 1 << (Self::BITS_L2 - 2);
    const BYTES: usize = 1 << (Self::BITS_L2 - 3);

    fn from_le_bytes(bytes: Self::Type_array_u8) -> Self;
    fn from_be_bytes(bytes: Self::Type_array_u8) -> Self;

    fn to_le_bytes(self) -> Self::Type_array_u8;

    fn le_iter_u8_to_iter_self<I: Iterator<Item = u8>>(src: I) -> impl Iterator<Item = Self>;
    fn le_slice_u8_to_iter_self(src: &[u8]) -> impl Iterator<Item = Self>;
    fn slice_self_to_le_iter_u8(src: &[Self]) -> impl Iterator<Item = u8>;
}

//---------- Lossy conversions.

pub trait Trunc<T> { fn trunc(self) -> T; }
impl Trunc< u8   > for u8   { fn trunc(self) -> u8   { self         } }
impl Trunc< u16  > for u8   { fn trunc(self) -> u16  { self as u16  } }
impl Trunc< u32  > for u8   { fn trunc(self) -> u32  { self as u32  } }
impl Trunc< u64  > for u8   { fn trunc(self) -> u64  { self as u64  } }
impl Trunc< u128 > for u8   { fn trunc(self) -> u128 { self as u128 } }
impl Trunc< u8   > for u16  { fn trunc(self) -> u8   { self as u8   } }
impl Trunc< u16  > for u16  { fn trunc(self) -> u16  { self         } }
impl Trunc< u32  > for u16  { fn trunc(self) -> u32  { self as u32  } }
impl Trunc< u64  > for u16  { fn trunc(self) -> u64  { self as u64  } }
impl Trunc< u128 > for u16  { fn trunc(self) -> u128 { self as u128 } }
impl Trunc< u8   > for u32  { fn trunc(self) -> u8   { self as u8   } }
impl Trunc< u16  > for u32  { fn trunc(self) -> u16  { self as u16  } }
impl Trunc< u32  > for u32  { fn trunc(self) -> u32  { self         } }
impl Trunc< u64  > for u32  { fn trunc(self) -> u64  { self as u64  } }
impl Trunc< u128 > for u32  { fn trunc(self) -> u128 { self as u128 } }
impl Trunc< u8   > for u64  { fn trunc(self) -> u8   { self as u8   } }
impl Trunc< u16  > for u64  { fn trunc(self) -> u16  { self as u16  } }
impl Trunc< u32  > for u64  { fn trunc(self) -> u32  { self as u32  } }
impl Trunc< u64  > for u64  { fn trunc(self) -> u64  { self         } }
impl Trunc< u128 > for u64  { fn trunc(self) -> u128 { self as u128 } }
impl Trunc< u8   > for u128 { fn trunc(self) -> u8   { self as u8   } }
impl Trunc< u16  > for u128 { fn trunc(self) -> u16  { self as u16  } }
impl Trunc< u32  > for u128 { fn trunc(self) -> u32  { self as u32  } }
impl Trunc< u64  > for u128 { fn trunc(self) -> u64  { self as u64  } }
impl Trunc< u128 > for u128 { fn trunc(self) -> u128 { self         } }

//---------- `AtMost` traits

pub trait PrimitiveUnsignedAtMost128: PrimitiveUnsigned {}
pub trait PrimitiveUnsignedAtMost64:
    PrimitiveUnsignedAtMost128 + std::convert::Into<u64> {}
pub trait PrimitiveUnsignedAtMost32: PrimitiveUnsignedAtMost64 + std::convert::Into<u32> {}
pub trait PrimitiveUnsignedAtMost16: PrimitiveUnsignedAtMost32 + std::convert::Into<u16> {}
pub trait PrimitiveUnsignedAtMost8: PrimitiveUnsignedAtMost16 + std::convert::Into<u8> {}

//---------- `AtLeast` traits

pub trait PrimitiveUnsignedAtLeast8: PrimitiveUnsigned {}
pub trait PrimitiveUnsignedAtLeast16: PrimitiveUnsignedAtLeast8 + std::convert::From<u16> {}
pub trait PrimitiveUnsignedAtLeast32: PrimitiveUnsignedAtLeast16 + std::convert::From<u32> {}
pub trait PrimitiveUnsignedAtLeast64: PrimitiveUnsignedAtLeast32 + std::convert::From<u64> {}
pub trait PrimitiveUnsignedAtLeast128:
    PrimitiveUnsignedAtLeast64 + std::convert::From<u128> { }

//---------- `Exactly` traits

pub trait PrimitiveUnsignedExactly_u8: PrimitiveUnsignedAtLeast8 + PrimitiveUnsignedAtMost8 {}
pub trait PrimitiveUnsignedExactly_u16: PrimitiveUnsignedAtLeast16 + PrimitiveUnsignedAtMost16 {}
pub trait PrimitiveUnsignedExactly_u32: PrimitiveUnsignedAtLeast32 + PrimitiveUnsignedAtMost32 {}
pub trait PrimitiveUnsignedExactly_u64: PrimitiveUnsignedAtLeast64 + PrimitiveUnsignedAtMost64 {}
pub trait PrimitiveUnsignedExactly_u128: PrimitiveUnsignedAtLeast128 + PrimitiveUnsignedAtMost128 { }

//------ impls on concrete types

//------ u8

impl PrimitiveUnsigned for u8 {
    type Type = u8;
    type Type_array_u8 = [u8; Self::SIZE];
    const NAME: &'static str = "u8";
    const BITS_L2: u8 = 3;
    //const ZERO: <Self as PrimitiveType>::PrimitiveType = 0;
    const ZERO: Self = 0;
    const ZERO_array_u8: Self::Type_array_u8 = [0_u8; Self::SIZE];
    const ONE: Self = 1;
    const MAX: Self = u8::MAX;

    fn from_le_bytes(bytes: Self::Type_array_u8) -> Self { bytes[0] }
    fn from_be_bytes(bytes: Self::Type_array_u8) -> Self { bytes[0] }

    fn le_iter_u8_to_iter_self<I: Iterator<Item = u8>>(src: I) -> impl Iterator<Item = Self> {
        src
    }

    fn le_slice_u8_to_iter_self(src: &[u8]) -> impl Iterator<Item = Self> {
        src.iter().copied()
    }

    fn to_le_bytes(self) -> Self::Type_array_u8 { [self] }

    fn slice_self_to_le_iter_u8(src: &[Self]) -> impl Iterator<Item = u8> {
        src.iter().copied()
    }
}

impl PrimitiveUnsignedAtLeast8 for u8 {}

impl PrimitiveUnsignedExactly_u8 for u8 {}

impl PrimitiveUnsignedAtMost8 for u8 {}
impl PrimitiveUnsignedAtMost16 for u8 {}
impl PrimitiveUnsignedAtMost32 for u8 {}
impl PrimitiveUnsignedAtMost64 for u8 {}
impl PrimitiveUnsignedAtMost128 for u8 {}

//------ u16

impl PrimitiveUnsigned for u16 {
    type Type = u16;
    type Type_array_u8 = [u8; Self::SIZE];
    const NAME: &'static str = "u16";
    const BITS_L2: u8 = 4;
    //const ZERO: <Self as PrimitiveUnsigned>::Type = 0;
    const ZERO: Self = 0;
    const ZERO_array_u8: Self::Type_array_u8 = [0_u8; Self::SIZE];
    const ONE: Self = 1;
    const MAX: Self = u16::MAX;

    fn from_le_bytes(bytes: Self::Type_array_u8) -> Self { u16::from_le_bytes(bytes) }
    fn from_be_bytes(bytes: Self::Type_array_u8) -> Self { u16::from_be_bytes(bytes) }

    #[allow(clippy::needless_range_loop)] //? TODO clippy
    fn le_iter_u8_to_iter_self<I: Iterator<Item = u8>>(src: I) -> impl Iterator<Item = Self> {
        const SIZE: usize = 2; // Shouldn't have to do this.
        let mut iter = src;
        let mut done = false;
        std::iter::from_fn(move || {
            if !done {
                let mut aby = [0u8; SIZE];
                for ix in 0..SIZE {
                    let opt_next = if !done { iter.next() } else { None };
                    if let Some(by) = opt_next {
                        aby[ix] = by;
                    } else {
                        aby[ix] = 0;
                        done = true;
                    }
                }
                Some(Self::from_le_bytes(aby))
            } else {
                None
            }
        })
    }

    fn le_slice_u8_to_iter_self(src: &[u8]) -> impl Iterator<Item = Self> {
        const SIZE: usize = 2; // Shouldn't have to do this.
        src.chunks(SIZE).map(|chunk| {
            let mut aby = [0u8; SIZE];
            if chunk.len() == SIZE {
                aby.copy_from_slice(chunk)
            } else {
                for (ix, &by) in chunk.iter().enumerate() {
                    aby[ix] = by;
                }
            }
            Self::from_le_bytes(aby)
        })
    }

    fn to_le_bytes(self) -> Self::Type_array_u8 { u16::to_le_bytes(self) }

    fn slice_self_to_le_iter_u8(src: &[Self]) -> impl Iterator<Item = u8> {
        src.iter().copied()
        .map(|e| Self::to_le_bytes(e).into_iter())
        .flatten()
    }
}

impl PrimitiveUnsignedAtLeast8 for u16 {}
impl PrimitiveUnsignedAtLeast16 for u16 {}

impl PrimitiveUnsignedExactly_u16 for u16 {}

impl PrimitiveUnsignedAtMost16 for u16 {}
impl PrimitiveUnsignedAtMost32 for u16 {}
impl PrimitiveUnsignedAtMost64 for u16 {}
impl PrimitiveUnsignedAtMost128 for u16 {}

//------ u32

impl PrimitiveUnsigned for u32 {
    type Type = u32;
    type Type_array_u8 = [u8; Self::SIZE];
    const NAME: &'static str = "u32";
    const BITS_L2: u8 = 5;
    const ZERO: Self = 0;
    const ZERO_array_u8: Self::Type_array_u8 = [0_u8; Self::SIZE];
    const ONE: Self = 1;
    const MAX: Self = u32::MAX;

    fn from_le_bytes(bytes: Self::Type_array_u8) -> Self { u32::from_le_bytes(bytes) }
    fn from_be_bytes(bytes: Self::Type_array_u8) -> Self { u32::from_be_bytes(bytes) }

    #[allow(clippy::needless_range_loop)] //? TODO clippy
    fn le_iter_u8_to_iter_self<I: Iterator<Item = u8>>(src: I) -> impl Iterator<Item = Self> {
        const SIZE: usize = 4; // Shouldn't have to do this.
        let mut iter = src;
        let mut done = false;
        std::iter::from_fn(move || {
            if !done {
                let mut aby = [0u8; SIZE];
                for ix in 0..SIZE {
                    let opt_next = if !done { iter.next() } else { None };
                    if let Some(by) = opt_next {
                        aby[ix] = by;
                    } else {
                        aby[ix] = 0;
                        done = true;
                    }
                }
                Some(Self::from_le_bytes(aby))
            } else {
                None
            }
        })
    }

    fn le_slice_u8_to_iter_self(src: &[u8]) -> impl Iterator<Item = Self> {
        const SIZE: usize = 4; // Shouldn't have to do this.
        src.chunks(SIZE).map(|chunk| {
            let mut aby = [0u8; SIZE];
            if chunk.len() == SIZE {
                aby.copy_from_slice(chunk)
            } else {
                for (ix, &by) in chunk.iter().enumerate() {
                    aby[ix] = by;
                }
            }
            Self::from_le_bytes(aby)
        })
    }

    fn to_le_bytes(self) -> Self::Type_array_u8 { u32::to_le_bytes(self) }

    fn slice_self_to_le_iter_u8(src: &[Self]) -> impl Iterator<Item = u8> {
        src.iter().copied()
        .map(|e| Self::to_le_bytes(e).into_iter())
        .flatten()
    }
}

impl PrimitiveUnsignedAtLeast8 for u32 {}
impl PrimitiveUnsignedAtLeast16 for u32 {}
impl PrimitiveUnsignedAtLeast32 for u32 {}

impl PrimitiveUnsignedExactly_u32 for u32 {}

impl PrimitiveUnsignedAtMost32 for u32 {}
impl PrimitiveUnsignedAtMost64 for u32 {}
impl PrimitiveUnsignedAtMost128 for u32 {}

//------ u64

impl PrimitiveUnsigned for u64 {
    type Type = u64;
    type Type_array_u8 = [u8; Self::SIZE];
    const NAME: &'static str = "u64";
    const BITS_L2: u8 = 6;
    const ZERO: Self = 0;
    const ZERO_array_u8: Self::Type_array_u8 = [0_u8; Self::SIZE];
    const ONE: Self = 1;
    const MAX: Self = u64::MAX;

    fn from_le_bytes(bytes: Self::Type_array_u8) -> Self { u64::from_le_bytes(bytes) }
    fn from_be_bytes(bytes: Self::Type_array_u8) -> Self { u64::from_be_bytes(bytes) }

    #[allow(clippy::needless_range_loop)] //? TODO clippy
    fn le_iter_u8_to_iter_self<I: Iterator<Item = u8>>(src: I) -> impl Iterator<Item = Self> {
        const SIZE: usize = 8; // Shouldn't have to do this.
        let mut iter = src;
        let mut done = false;
        std::iter::from_fn(move || {
            if !done {
                let mut aby = [0u8; SIZE];
                for ix in 0..SIZE {
                    let opt_next = if !done { iter.next() } else { None };
                    if let Some(by) = opt_next {
                        aby[ix] = by;
                    } else {
                        aby[ix] = 0;
                        done = true;
                    }
                }
                Some(Self::from_le_bytes(aby))
            } else {
                None
            }
        })
    }

    fn le_slice_u8_to_iter_self(src: &[u8]) -> impl Iterator<Item = Self> {
        const SIZE: usize = 8; // Shouldn't have to do this.
        src.chunks(SIZE).map(|chunk| {
            let mut aby = [0u8; SIZE];
            if chunk.len() == SIZE {
                aby.copy_from_slice(chunk)
            } else {
                for (ix, &by) in chunk.iter().enumerate() {
                    aby[ix] = by;
                }
            }
            Self::from_le_bytes(aby)
        })
    }

    fn to_le_bytes(self) -> Self::Type_array_u8 { u64::to_le_bytes(self) }

    fn slice_self_to_le_iter_u8(src: &[Self]) -> impl Iterator<Item = u8> {
        src.iter().copied()
        .map(|e| Self::to_le_bytes(e).into_iter())
        .flatten()
    }
}

impl PrimitiveUnsignedAtLeast8 for u64 {}
impl PrimitiveUnsignedAtLeast16 for u64 {}
impl PrimitiveUnsignedAtLeast32 for u64 {}
impl PrimitiveUnsignedAtLeast64 for u64 {}

impl PrimitiveUnsignedExactly_u64 for u64 {}

impl PrimitiveUnsignedAtMost64 for u64 {}
impl PrimitiveUnsignedAtMost128 for u64 {}

//------ u128

impl PrimitiveUnsigned for u128 {
    type Type = u128;
    type Type_array_u8 = [u8; Self::SIZE];
    const NAME: &'static str = "u128";
    const BITS_L2: u8 = 7;
    const ZERO: Self = 0;
    const ZERO_array_u8: Self::Type_array_u8 = [0_u8; Self::SIZE];
    const ONE: Self = 1;
    const MAX: Self = u128::MAX;

    fn from_le_bytes(bytes: Self::Type_array_u8) -> Self { u128::from_le_bytes(bytes) }
    fn from_be_bytes(bytes: Self::Type_array_u8) -> Self { u128::from_be_bytes(bytes) }

    fn le_iter_u8_to_iter_self<I: Iterator<Item = u8>>(src: I) -> impl Iterator<Item = Self> {
        const SIZE: usize = 16; // Shouldn't have to do this.
        let mut iter = src;
        let mut done = false;
        std::iter::from_fn(move || {
            if !done {
                let mut aby = [0u8; SIZE];
                for ix in 0..SIZE {
                    let opt_next = if !done { iter.next() } else { None };
                    if let Some(by) = opt_next {
                        aby[ix] = by;
                    } else {
                        aby[ix] = 0;
                        done = true;
                    }
                }
                Some(Self::from_le_bytes(aby))
            } else {
                None
            }
        })
    }

    fn le_slice_u8_to_iter_self(src: &[u8]) -> impl Iterator<Item = Self> {
        const SIZE: usize = 16; // Shouldn't have to do this.
        src.chunks(SIZE).map(|chunk| {
            let mut aby = [0u8; SIZE];
            if chunk.len() == SIZE {
                aby.copy_from_slice(chunk)
            } else {
                for (ix, &by) in chunk.iter().enumerate() {
                    aby[ix] = by;
                }
            }
            Self::from_le_bytes(aby)
        })
    }

    fn to_le_bytes(self) -> Self::Type_array_u8 { u128::to_le_bytes(self) }

    fn slice_self_to_le_iter_u8(src: &[Self]) -> impl Iterator<Item = u8> {
        src.iter().copied()
        .map(|e| Self::to_le_bytes(e).into_iter())
        .flatten()
    }
}

impl PrimitiveUnsignedAtLeast8 for u128 {}
impl PrimitiveUnsignedAtLeast16 for u128 {}
impl PrimitiveUnsignedAtLeast32 for u128 {}
impl PrimitiveUnsignedAtLeast64 for u128 {}
impl PrimitiveUnsignedAtLeast128 for u128 {}

impl PrimitiveUnsignedExactly_u128 for u128 {}

impl PrimitiveUnsignedAtMost128 for u128 {}

//=================================================================================================|

/*
pub trait WiderOf { type Type; }
impl WiderOf for (u8, u8  ) { type Type = u8; }
impl WiderOf for (u8, u16 ) { type Type = u16; }
impl WiderOf for (u8, u32 ) { type Type = u32; }
impl WiderOf for (u8, u64 ) { type Type = u64; }
impl WiderOf for (u8, u128) { type Type = u128; }
impl WiderOf for (u16, u8  ) { type Type = u16; }
impl WiderOf for (u16, u16 ) { type Type = u16; }
impl WiderOf for (u16, u32 ) { type Type = u32; }
impl WiderOf for (u16, u64 ) { type Type = u64; }
impl WiderOf for (u16, u128) { type Type = u128; }
impl WiderOf for (u32, u8  ) { type Type = u32; }
impl WiderOf for (u32, u16 ) { type Type = u32; }
impl WiderOf for (u32, u32 ) { type Type = u32; }
impl WiderOf for (u32, u64 ) { type Type = u64; }
impl WiderOf for (u32, u128) { type Type = u128; }
impl WiderOf for (u64, u8  ) { type Type = u64; }
impl WiderOf for (u64, u16 ) { type Type = u64; }
impl WiderOf for (u64, u32 ) { type Type = u64; }
impl WiderOf for (u64, u64 ) { type Type = u64; }
impl WiderOf for (u64, u128) { type Type = u128; }
impl WiderOf for (u128, u8  ) { type Type = u128; }
impl WiderOf for (u128, u16 ) { type Type = u128; }
impl WiderOf for (u128, u32 ) { type Type = u128; }
impl WiderOf for (u128, u64 ) { type Type = u128; }
impl WiderOf for (u128, u128) { type Type = u128; }
*/

//---------- Selecting the next larger size.

pub trait NextLargerThan: PrimitiveUnsignedAtLeast8 + PrimitiveUnsignedAtMost64
{
    type Type: PrimitiveUnsignedAtLeast16 + PrimitiveUnsignedAtMost128;
}
impl NextLargerThan for u8   { type Type = u16;   }
impl NextLargerThan for u16  { type Type = u32;  }
impl NextLargerThan for u32  { type Type = u64;  }
impl NextLargerThan for u64  { type Type = u128; }
// But not for `u128`.

//=================================================================================================|

/// A trait allowing a function to accept any primitive unsigned parameter not larger than 64 bits.
///
/// It also accepts `usize`, so only use this if you're prepared to change caller's code on
/// platforms where a `usize` cannot losslessly be converted into a `u64`.
pub trait Number_Into_u64 {
    fn into_u64(self) -> u64;
}

impl Number_Into_u64 for usize {
    fn into_u64(self) -> u64 {
        debug_assert!(std::mem::size_of::<usize>() <= 8);
        self as u64
    }
}

impl<T> Number_Into_u64 for T where T: PrimitiveUnsignedAtMost64 {
    fn into_u64(self) -> u64 {
        self.into()
    }
}

//-------------------------------------------------------------------------------------------------|

/// A trait allowing a function to accept any primitive unsigned parameter not larger than 128 bits.
///
/// It also accepts `usize`, so only use this if you're prepared to change caller's code on
/// platforms where a `usize` cannot losslessly be converted into a `u128`.
pub trait Number_Into_u128 {
    fn into_u128(self) -> u128;
}

impl Number_Into_u128 for usize {
    fn into_u128(self) -> u128 {
        debug_assert!(std::mem::size_of::<usize>() <= 16);
        self as u128
    }
}

impl<T> Number_Into_u128 for T where T: PrimitiveUnsignedAtMost128 {
    fn into_u128(self) -> u128 {
        self.into()
    }
}

//=================================================================================================|

/// Expands the statements for each of `u8` through `u128`, defining the type alias
/// $UPT to each in turn.
#[macro_export]
macro_rules! for_each_fixed_width_unsigned_primitive_type {
    ($UPT:ident => $( $s:stmt );*) => {{
        { type $UPT = u8; $( $s );* }
        { type $UPT = u16; $( $s );* }
        { type $UPT = u32; $( $s );* }
        { type $UPT = u64; $( $s );* }
        { type $UPT = u128; $( $s );* }
    }};
}

//=================================================================================================|

#[cfg(test)]
mod tests {
    use super::*;

    #[inline(always)]
    #[must_use]
    fn pow2_minus_1_saturating<T: PrimitiveUnsigned>(n: usize) -> T {
        if n < T::BITS {
            (T::ONE << n) - T::ONE
        } else {
            T::MAX
        }
    }

    #[test]
    fn t00() {
        fn check_primitiveunsigned<T>()
        where
            T: PrimitiveUnsigned<Type = T>,
            T: std::convert::Into<u128>,
        {
            let size_of_t = std::mem::size_of::<T>() as u64;

            assert_eq!(T::SIZE as u64, size_of_t);
            assert_eq!(1 << T::BITS_L2, T::BITS as u64);
            assert_eq!(T::BITS as u64, size_of_t * 8);
            assert_eq!(T::ZERO, T::from(false));
            assert_eq!(T::ONE, T::from(true));

            assert!(PRIMITIVEUNSIGNED_BITS_L2_RANGE.contains(&T::BITS_L2));

            let check_primitiveunsigned_shift_count = |n: usize| {
                let t = pow2_minus_1_saturating::<T>(n);
                let t128 = Into::<u128>::into(t);
                //eprintln!("t128={:b}", t128);
                let expected_ones: usize = n.min(T::BITS);
                assert_eq!(t128.count_ones() as usize, expected_ones);
                if 0 < n {
                    assert_eq!(t & T::ONE, T::ONE);
                }
            };

            check_primitiveunsigned_shift_count(0);
            check_primitiveunsigned_shift_count(1);
            check_primitiveunsigned_shift_count(2);
            check_primitiveunsigned_shift_count(T::BITS - 2);
            check_primitiveunsigned_shift_count(T::BITS - 1);
            check_primitiveunsigned_shift_count(T::BITS);
            check_primitiveunsigned_shift_count(T::BITS + 1);
        }

        check_primitiveunsigned::<u8>();
        check_primitiveunsigned::<u16>();
        check_primitiveunsigned::<u32>();
        check_primitiveunsigned::<u64>();
        check_primitiveunsigned::<u128>();
        assert_eq!(PRIMITIVEUNSIGNED_BITS_MAX, 128);
    }

    #[cfg(feature = "num-integer")]
    #[test]
    fn t01() {
        fn take_expect_into_u128<T: Number_Into_u128>(m: T) {
            let q: u128 = m.into_u128();
            debug_assert_ne!(q, 0);
        }

        take_expect_into_u128(usize::MAX);

        for_each_fixed_width_unsigned_primitive_type!{ PrimitiveUnsignedType => {
            take_expect_into_u128(PrimitiveUnsignedType::MAX);
        }}
    }
}
