// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::assertions_on_constants)]

//=================================================================================================|

/// Returns the smallest of a sequence of `u8` values, or `0` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_min_u8(mut slice: &[u8]) -> u8 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if *fst < acc {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        0
    }
}

/// Returns the largest of a sequence of `u8` values, or `u8::MAX` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_max_u8(mut slice: &[u8]) -> u8 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if acc < *fst {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        u8::MAX
    }
}

//=================================================================================================|

/// Returns the smallest of a sequence of `u16` values, or `0` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_min_u16(mut slice: &[u16]) -> u16 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if *fst < acc {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        0
    }
}

/// Returns the largest of a sequence of `u16` values, or `u16::MAX` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_max_u16(mut slice: &[u16]) -> u16 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if acc < *fst {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        u16::MAX
    }
}

//=================================================================================================|

/// Returns the smallest of a sequence of `u32` values, or `0` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_min_u32(mut slice: &[u32]) -> u32 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if *fst < acc {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        0
    }
}

/// Returns the largest of a sequence of `u32` values, or `u32::MAX` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_max_u32(mut slice: &[u32]) -> u32 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if acc < *fst {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        u32::MAX
    }
}

//=================================================================================================|

/// Returns the smallest of a sequence of `usize` values, or `0` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_min_usize(mut slice: &[usize]) -> usize {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if *fst < acc {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        0
    }
}

/// Returns the largest of a sequence of `usize` values, or  `usize::MAX` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_max_usize(mut slice: &[usize]) -> usize {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if acc < *fst {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        usize::MAX
    }
}

//=================================================================================================|

/// Returns the smallest of a sequence of `u64` values, or `0` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_min_u64(mut slice: &[u64]) -> u64 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if *fst < acc {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        0
    }
}

/// Returns the largest of a sequence of `u64` values, or `u64::MAX` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_max_u64(mut slice: &[u64]) -> u64 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if acc < *fst {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        u64::MAX
    }
}

//=================================================================================================|

/// Returns the smallest of a sequence of `u128` values, or `0` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_min_u128(mut slice: &[u128]) -> u128 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if *fst < acc {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        0
    }
}

/// Returns the largest of a sequence of `u128` values, or `u128::MAX` if the sequence is empty.
///
/// FIXME(const-hack): Someday Rust `std` will provide this.
/// https://github.com/rust-lang/rust/issues/57563
pub const fn const_max_u128(mut slice: &[u128]) -> u128 {
    if let &[mut acc, ref rest @ ..] = slice {
        slice = rest;
        while let [fst, rest @ ..] = slice {
            if acc < *fst {
                acc = *fst;
            }
            slice = rest;
        }
        acc
    } else {
        u128::MAX
    }
}

//=================================================================================================|

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod t {
    use super::*;
    use static_assertions::const_assert_eq;

    #[test]
    fn t1_u8() {
        //- min

        const_assert_eq!(const_min_u8(&[]), 0);

        const_assert_eq!(const_min_u8(&[0]), 0);
        const_assert_eq!(const_min_u8(&[1]), 1);

        const_assert_eq!(const_min_u8(&[0, 1]), 0);
        const_assert_eq!(const_min_u8(&[3, 2]), 2);

        const_assert_eq!(const_min_u8(&[5, 6, 7]), 5);
        const_assert_eq!(const_min_u8(&[11, 10, 12]), 10);
        const_assert_eq!(const_min_u8(&[15, 14, 13]), 13);

        //- max

        const_assert_eq!(const_max_u8(&[]), u8::MAX);

        const_assert_eq!(const_max_u8(&[0]), 0);
        const_assert_eq!(const_max_u8(&[1]), 1);

        const_assert_eq!(const_max_u8(&[0, 1]), 1);
        const_assert_eq!(const_max_u8(&[1, 0]), 1);
        const_assert_eq!(const_max_u8(&[1, 1]), 1);
        const_assert_eq!(const_max_u8(&[3, 2]), 3);
        const_assert_eq!(const_max_u8(&[1, u8::MAX]), u8::MAX);

        const_assert_eq!(const_max_u8(&[7, 5, 6]), 7);
        const_assert_eq!(const_max_u8(&[11, 12, 10]), 12);
        const_assert_eq!(const_max_u8(&[14, 13, 15]), 15);
    }

    #[test]
    fn t2_u16() {
        //- min

        const_assert_eq!(const_min_u16(&[]), 0);

        const_assert_eq!(const_min_u16(&[0]), 0);
        const_assert_eq!(const_min_u16(&[1]), 1);

        const_assert_eq!(const_min_u16(&[0, 1]), 0);
        const_assert_eq!(const_min_u16(&[3, 2]), 2);

        const_assert_eq!(const_min_u16(&[5, 6, 7]), 5);
        const_assert_eq!(const_min_u16(&[11, 10, 12]), 10);
        const_assert_eq!(const_min_u16(&[15, 14, 13]), 13);

        //- max

        const_assert_eq!(const_max_u16(&[]), u16::MAX);

        const_assert_eq!(const_max_u16(&[0]), 0);
        const_assert_eq!(const_max_u16(&[1]), 1);

        const_assert_eq!(const_max_u16(&[0, 1]), 1);
        const_assert_eq!(const_max_u16(&[1, 0]), 1);
        const_assert_eq!(const_max_u16(&[1, 1]), 1);
        const_assert_eq!(const_max_u16(&[3, 2]), 3);
        const_assert_eq!(const_max_u16(&[1, u16::MAX]), u16::MAX);

        const_assert_eq!(const_max_u16(&[7, 5, 6]), 7);
        const_assert_eq!(const_max_u16(&[11, 12, 10]), 12);
        const_assert_eq!(const_max_u16(&[14, 13, 15]), 15);
    }

    #[test]
    fn t3_u32() {
        //- min

        const_assert_eq!(const_min_u32(&[]), 0);

        const_assert_eq!(const_min_u32(&[0]), 0);
        const_assert_eq!(const_min_u32(&[1]), 1);

        const_assert_eq!(const_min_u32(&[0, 1]), 0);
        const_assert_eq!(const_min_u32(&[3, 2]), 2);

        const_assert_eq!(const_min_u32(&[5, 6, 7]), 5);
        const_assert_eq!(const_min_u32(&[11, 10, 12]), 10);
        const_assert_eq!(const_min_u32(&[15, 14, 13]), 13);

        //- max

        const_assert_eq!(const_max_u32(&[]), u32::MAX);

        const_assert_eq!(const_max_u32(&[0]), 0);
        const_assert_eq!(const_max_u32(&[1]), 1);

        const_assert_eq!(const_max_u32(&[0, 1]), 1);
        const_assert_eq!(const_max_u32(&[1, 0]), 1);
        const_assert_eq!(const_max_u32(&[1, 1]), 1);
        const_assert_eq!(const_max_u32(&[3, 2]), 3);
        const_assert_eq!(const_max_u32(&[1, u32::MAX]), u32::MAX);

        const_assert_eq!(const_max_u32(&[7, 5, 6]), 7);
        const_assert_eq!(const_max_u32(&[11, 12, 10]), 12);
        const_assert_eq!(const_max_u32(&[14, 13, 15]), 15);
    }

    #[test]
    fn t4_usize() {
        //- min

        const_assert_eq!(const_min_usize(&[]), 0);

        const_assert_eq!(const_min_usize(&[0]), 0);
        const_assert_eq!(const_min_usize(&[1]), 1);

        const_assert_eq!(const_min_usize(&[0, 1]), 0);
        const_assert_eq!(const_min_usize(&[1, 0]), 0);
        const_assert_eq!(const_min_usize(&[1, 1]), 1);
        const_assert_eq!(const_min_usize(&[3, 2]), 2);
        const_assert_eq!(const_min_usize(&[1, usize::MAX]), 1);

        const_assert_eq!(const_min_usize(&[5, 6, 7]), 5);
        const_assert_eq!(const_min_usize(&[11, 10, 12]), 10);
        const_assert_eq!(const_min_usize(&[15, 14, 13]), 13);

        //- max

        const_assert_eq!(const_max_usize(&[]), usize::MAX);

        const_assert_eq!(const_max_usize(&[0]), 0);
        const_assert_eq!(const_max_usize(&[1]), 1);

        const_assert_eq!(const_max_usize(&[0, 1]), 1);
        const_assert_eq!(const_max_usize(&[1, 0]), 1);
        const_assert_eq!(const_max_usize(&[1, 1]), 1);
        const_assert_eq!(const_max_usize(&[3, 2]), 3);
        const_assert_eq!(const_max_usize(&[1, usize::MAX]), usize::MAX);

        const_assert_eq!(const_max_usize(&[7, 5, 6]), 7);
        const_assert_eq!(const_max_usize(&[11, 12, 10]), 12);
        const_assert_eq!(const_max_usize(&[14, 13, 15]), 15);
    }

    #[test]
    fn t5_u64() {
        //- min

        const_assert_eq!(const_min_u64(&[]), 0);

        const_assert_eq!(const_min_u64(&[0]), 0);
        const_assert_eq!(const_min_u64(&[1]), 1);

        const_assert_eq!(const_min_u64(&[0, 1]), 0);
        const_assert_eq!(const_min_u64(&[3, 2]), 2);

        const_assert_eq!(const_min_u64(&[5, 6, 7]), 5);
        const_assert_eq!(const_min_u64(&[11, 10, 12]), 10);
        const_assert_eq!(const_min_u64(&[15, 14, 13]), 13);

        //- max

        const_assert_eq!(const_max_u64(&[]), u64::MAX);

        const_assert_eq!(const_max_u64(&[0]), 0);
        const_assert_eq!(const_max_u64(&[1]), 1);

        const_assert_eq!(const_max_u64(&[0, 1]), 1);
        const_assert_eq!(const_max_u64(&[1, 0]), 1);
        const_assert_eq!(const_max_u64(&[1, 1]), 1);
        const_assert_eq!(const_max_u64(&[3, 2]), 3);
        const_assert_eq!(const_max_u64(&[1, u64::MAX]), u64::MAX);

        const_assert_eq!(const_max_u64(&[7, 5, 6]), 7);
        const_assert_eq!(const_max_u64(&[11, 12, 10]), 12);
        const_assert_eq!(const_max_u64(&[14, 13, 15]), 15);
    }

    #[test]
    fn t6_u128() {
        //- min

        const_assert_eq!(const_min_u128(&[]), 0);

        const_assert_eq!(const_min_u128(&[0]), 0);
        const_assert_eq!(const_min_u128(&[1]), 1);

        const_assert_eq!(const_min_u128(&[0, 1]), 0);
        const_assert_eq!(const_min_u128(&[3, 2]), 2);

        const_assert_eq!(const_min_u128(&[5, 6, 7]), 5);
        const_assert_eq!(const_min_u128(&[11, 10, 12]), 10);
        const_assert_eq!(const_min_u128(&[15, 14, 13]), 13);

        //- max

        const_assert_eq!(const_max_u128(&[]), u128::MAX);

        const_assert_eq!(const_max_u128(&[0]), 0);
        const_assert_eq!(const_max_u128(&[1]), 1);

        const_assert_eq!(const_max_u128(&[0, 1]), 1);
        const_assert_eq!(const_max_u128(&[1, 0]), 1);
        const_assert_eq!(const_max_u128(&[1, 1]), 1);
        const_assert_eq!(const_max_u128(&[3, 2]), 3);
        const_assert_eq!(const_max_u128(&[1, u128::MAX]), u128::MAX);

        const_assert_eq!(const_max_u128(&[7, 5, 6]), 7);
        const_assert_eq!(const_max_u128(&[11, 12, 10]), 12);
        const_assert_eq!(const_max_u128(&[14, 13, 15]), 15);
    }
}
