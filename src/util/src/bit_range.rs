// Copyright (C) Microsoft Corporation. All rights reserved.

//#![cfg_attr(rustfmt, rustfmt_skip)]
#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(elided_lifetimes_in_paths)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

use num_traits::{ConstOne, ConstZero, PrimInt, Unsigned, WrappingAdd};

//=================================================================================================|

/// For each range of bit lengths `1..=size_of::<T>()*8`, calls the supplied function with first
/// and last values, and number of bits.
pub fn foreach_bit_length_range<T, F>(max_bits: u32, f: F)
where
    T: PrimInt + Unsigned + ConstZero + ConstOne + WrappingAdd,
    F: FnMut(T, T, u32),
{
    let t_bits = T::ZERO.count_zeros();
    let t_all_ones = !T::ZERO;

    let max_bits = t_bits.min(max_bits);

    let last = t_all_ones >> (t_bits - max_bits) as usize;

    foreach_bit_length_range_optfirst_optlast::<T, F>(None, Some(last), f)
}

//-------------------------------------------------------------------------------------------------|

/// For each range of bit lengths `1..=size_of::<T>()*8`, calls the supplied function with first
/// and last values, and number of bits.
pub fn foreach_bit_length_range_optfirst_optlast<T, F>(
    opt_first: Option<T>,
    opt_last: Option<T>,
    mut f: F,
) where
    T: PrimInt + Unsigned + ConstZero + ConstOne + WrappingAdd,
    F: FnMut(T, T, u32),
{
    let t_bits = T::ZERO.count_zeros();
    let t_all_ones = !T::ZERO;

    let (mut first, first_bits) = if let Some(first) = opt_first {
        let leading_zeroes = (first | T::ONE).leading_zeros();
        let first_bits = t_bits - leading_zeroes;
        (first, first_bits)
    } else {
        (T::ZERO, 1_u32)
    };

    let (last, last_bits) = if let Some(last) = opt_last {
        let leading_zeroes = (last | T::ONE).leading_zeros();
        let first_bits = t_bits - leading_zeroes;
        (last, first_bits)
    } else {
        (t_all_ones, t_bits)
    };

    if first <= last {
        for bits in first_bits..=last_bits {
            let last: T = if bits < last_bits {
                t_all_ones >> (t_bits - bits) as usize
            } else {
                last
            };

            f(first, last, bits);

            first = last.wrapping_add(&T::ONE);
        }
    }
}

//=================================================================================================|

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::write_literal
)]
mod t {
    use super::*;
    use insta::assert_snapshot;
    use itertools::Itertools;

    enum FnUnderTestArgs<T> {
        OptOpt {
            opt_first: Option<T>,
            opt_last: Option<T>,
        },
        MaxBits(u32),
    }

    fn t_lines<T>(fn_under_test: FnUnderTestArgs<T>) -> String
    where
        T: std::fmt::UpperHex
            + std::fmt::Binary
            + std::fmt::Debug
            + std::fmt::Display
            + PrimInt
            + Unsigned
            + ConstZero
            + ConstOne
            + WrappingAdd,
    {
        use std::io::{Cursor, Write};
        let t_bits = T::ZERO.count_zeros();

        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::<u8>::with_capacity(8000));

        writeln!(buf, "T bits:    {t_bits}").unwrap();

        let f = |buf: &mut Cursor<Vec<u8>>, first: T, last: T, _bits: u32| {
            writeln!(buf, "----76543210--------------------------------").unwrap();
            writeln!(
                buf,
                "    {n:8b}  {n:#04X}  {n:3} {s}",
                n = first,
                s = "first"
            )
            .unwrap();
            writeln!(buf, "..= {n:8b}  {n:#04X}  {n:3} {s}", n = last, s = "last").unwrap();
        };

        match fn_under_test {
            FnUnderTestArgs::OptOpt {
                opt_first,
                opt_last,
            } => {
                writeln!(buf, "opt_first: {opt_first:?}").unwrap();
                writeln!(buf, "opt_last : {opt_last:?}").unwrap();
                foreach_bit_length_range_optfirst_optlast(
                    opt_first,
                    opt_last,
                    |first: T, last: T, bits: u32| f(&mut buf, first, last, bits),
                );
            }
            FnUnderTestArgs::MaxBits(max_bits) => {
                writeln!(buf, "max_bits : {max_bits:?}").unwrap();
                foreach_bit_length_range::<T, _>(max_bits, |first: T, last: T, bits: u32| {
                    f(&mut buf, first, last, bits)
                });
            }
        }

        writeln!(buf, "----76543210--------------------------------").unwrap();

        String::from_utf8(buf.into_inner())
            .unwrap()
            .lines()
            .join("\n")
    }

    #[test]
    fn t1() {
        let fn_under_test_args = FnUnderTestArgs::<u8>::OptOpt {
            opt_first: Some(0x_15),
            opt_last: Some(0x_E9),
        };
        assert_snapshot!(
            t_lines(fn_under_test_args),
            @r#"
        T bits:    8
        opt_first: Some(21)
        opt_last : Some(233)
        ----76543210--------------------------------
               10101  0x15   21 first
        ..=    11111  0x1F   31 last
        ----76543210--------------------------------
              100000  0x20   32 first
        ..=   111111  0x3F   63 last
        ----76543210--------------------------------
             1000000  0x40   64 first
        ..=  1111111  0x7F  127 last
        ----76543210--------------------------------
            10000000  0x80  128 first
        ..= 11101001  0xE9  233 last
        ----76543210--------------------------------
        "#);
    }

    #[test]
    fn t2() {
        let fn_under_test_args = FnUnderTestArgs::<u16>::OptOpt {
            opt_first: Some(0x3333_u16),
            opt_last: None,
        };
        assert_snapshot!(
            t_lines(fn_under_test_args),
            @r#"
        T bits:    16
        opt_first: Some(13107)
        opt_last : None
        ----76543210--------------------------------
            11001100110011  0x3333  13107 first
        ..= 11111111111111  0x3FFF  16383 last
        ----76543210--------------------------------
            100000000000000  0x4000  16384 first
        ..= 111111111111111  0x7FFF  32767 last
        ----76543210--------------------------------
            1000000000000000  0x8000  32768 first
        ..= 1111111111111111  0xFFFF  65535 last
        ----76543210--------------------------------
        "#);
    }

    #[test]
    fn t3() {
        let fn_under_test_args = FnUnderTestArgs::<u32>::OptOpt {
            opt_first: Some(0x_00001234_u32),
            opt_last: Some(0x_C7654321_u32),
        };
        assert_snapshot!(
            t_lines(fn_under_test_args),
            @r#"
        T bits:    32
        opt_first: Some(4660)
        opt_last : Some(3345302305)
        ----76543210--------------------------------
            1001000110100  0x1234  4660 first
        ..= 1111111111111  0x1FFF  8191 last
        ----76543210--------------------------------
            10000000000000  0x2000  8192 first
        ..= 11111111111111  0x3FFF  16383 last
        ----76543210--------------------------------
            100000000000000  0x4000  16384 first
        ..= 111111111111111  0x7FFF  32767 last
        ----76543210--------------------------------
            1000000000000000  0x8000  32768 first
        ..= 1111111111111111  0xFFFF  65535 last
        ----76543210--------------------------------
            10000000000000000  0x10000  65536 first
        ..= 11111111111111111  0x1FFFF  131071 last
        ----76543210--------------------------------
            100000000000000000  0x20000  131072 first
        ..= 111111111111111111  0x3FFFF  262143 last
        ----76543210--------------------------------
            1000000000000000000  0x40000  262144 first
        ..= 1111111111111111111  0x7FFFF  524287 last
        ----76543210--------------------------------
            10000000000000000000  0x80000  524288 first
        ..= 11111111111111111111  0xFFFFF  1048575 last
        ----76543210--------------------------------
            100000000000000000000  0x100000  1048576 first
        ..= 111111111111111111111  0x1FFFFF  2097151 last
        ----76543210--------------------------------
            1000000000000000000000  0x200000  2097152 first
        ..= 1111111111111111111111  0x3FFFFF  4194303 last
        ----76543210--------------------------------
            10000000000000000000000  0x400000  4194304 first
        ..= 11111111111111111111111  0x7FFFFF  8388607 last
        ----76543210--------------------------------
            100000000000000000000000  0x800000  8388608 first
        ..= 111111111111111111111111  0xFFFFFF  16777215 last
        ----76543210--------------------------------
            1000000000000000000000000  0x1000000  16777216 first
        ..= 1111111111111111111111111  0x1FFFFFF  33554431 last
        ----76543210--------------------------------
            10000000000000000000000000  0x2000000  33554432 first
        ..= 11111111111111111111111111  0x3FFFFFF  67108863 last
        ----76543210--------------------------------
            100000000000000000000000000  0x4000000  67108864 first
        ..= 111111111111111111111111111  0x7FFFFFF  134217727 last
        ----76543210--------------------------------
            1000000000000000000000000000  0x8000000  134217728 first
        ..= 1111111111111111111111111111  0xFFFFFFF  268435455 last
        ----76543210--------------------------------
            10000000000000000000000000000  0x10000000  268435456 first
        ..= 11111111111111111111111111111  0x1FFFFFFF  536870911 last
        ----76543210--------------------------------
            100000000000000000000000000000  0x20000000  536870912 first
        ..= 111111111111111111111111111111  0x3FFFFFFF  1073741823 last
        ----76543210--------------------------------
            1000000000000000000000000000000  0x40000000  1073741824 first
        ..= 1111111111111111111111111111111  0x7FFFFFFF  2147483647 last
        ----76543210--------------------------------
            10000000000000000000000000000000  0x80000000  2147483648 first
        ..= 11000111011001010100001100100001  0xC7654321  3345302305 last
        ----76543210--------------------------------
        "#);
    }

    #[test]
    fn t4() {
        let fn_under_test_args = FnUnderTestArgs::<u64>::MaxBits(10);
        assert_snapshot!(
            t_lines(fn_under_test_args),
            @r#"
        T bits:    64
        max_bits : 10
        ----76543210--------------------------------
                   0  0x00    0 first
        ..=        1  0x01    1 last
        ----76543210--------------------------------
                  10  0x02    2 first
        ..=       11  0x03    3 last
        ----76543210--------------------------------
                 100  0x04    4 first
        ..=      111  0x07    7 last
        ----76543210--------------------------------
                1000  0x08    8 first
        ..=     1111  0x0F   15 last
        ----76543210--------------------------------
               10000  0x10   16 first
        ..=    11111  0x1F   31 last
        ----76543210--------------------------------
              100000  0x20   32 first
        ..=   111111  0x3F   63 last
        ----76543210--------------------------------
             1000000  0x40   64 first
        ..=  1111111  0x7F  127 last
        ----76543210--------------------------------
            10000000  0x80  128 first
        ..= 11111111  0xFF  255 last
        ----76543210--------------------------------
            100000000  0x100  256 first
        ..= 111111111  0x1FF  511 last
        ----76543210--------------------------------
            1000000000  0x200  512 first
        ..= 1111111111  0x3FF  1023 last
        ----76543210--------------------------------
        "#);
    }

    #[test]
    fn t5() {
        let fn_under_test_args = FnUnderTestArgs::<u128>::MaxBits(6);
        assert_snapshot!(
            t_lines(fn_under_test_args),
            @r#"
        T bits:    128
        max_bits : 6
        ----76543210--------------------------------
                   0  0x00    0 first
        ..=        1  0x01    1 last
        ----76543210--------------------------------
                  10  0x02    2 first
        ..=       11  0x03    3 last
        ----76543210--------------------------------
                 100  0x04    4 first
        ..=      111  0x07    7 last
        ----76543210--------------------------------
                1000  0x08    8 first
        ..=     1111  0x0F   15 last
        ----76543210--------------------------------
               10000  0x10   16 first
        ..=    11111  0x1F   31 last
        ----76543210--------------------------------
              100000  0x20   32 first
        ..=   111111  0x3F   63 last
        ----76543210--------------------------------
        "#);
    }
}
