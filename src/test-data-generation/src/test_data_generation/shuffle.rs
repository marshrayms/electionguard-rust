// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(elided_lifetimes_in_paths)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

use std::{hash::Hash, num::NonZeroU64};
#[rustfmt::skip] //? TODO: Remove temp development code
use std::{
    borrow::Cow,
    iter::zip,
    sync::Arc,
};

use const_default::ConstDefault;
use indoc::{eprintdoc, formatdoc, writedoc};
use rand::{Rng, RngCore, SeedableRng, distr::Uniform};

//
use util::const_minmax::const_max_u64;

//=================================================================================================|

fn fmt_u32_xb(n: u32) -> String {
    format!("0x_{n:08X}_u32={n:032b}={n}")
}

fn fmt_u64_xd(n: u64) -> String {
    format!("0x_{:08X}_{:08X}_u64={n}", (n >> 32) as u32, n as u32)
}

fn fmt_u64_x(n: u64) -> String {
    format!("0x_{:08X}_{:08X}_u64", (n >> 32) as u32, n as u32)
}

fn fmt_u128_x(n: u128) -> String {
    format!(
        "0x_{:08X}_{:08X}_{:08X}_{:08X}_u128",
        (n >> 96) as u32,
        (n >> 64) as u32,
        (n >> 32) as u32,
        n as u32
    )
}

fn fmt_u64_dx(n: u64) -> String {
    format!("{n} (0x{:08X}_{:08X})", (n >> 32) as u32, n as u32)
}

fn fmt_u64_xdw<N: Into<u64>>(n: N, wid: usize) -> String {
    let n: u64 = n.into();
    format!("0x_{:08X}_{:08X}_u64={n:wid$}", (n >> 32) as u32, n as u32)
}

//=================================================================================================|

/// Shuffle of some number (`1..=2^64`) elements with O(1) memory and O(1) lookup.
///
/// Some might call this a `permutation`, but that could imply more than is intended.
/// For example, only the forward transform is defined.
///
/// This is intended only for use for efficient generation of example test data. It should not be
/// used for anything requiring real statistical or cryptographic properties.
pub struct Shuffle<R>
where
    R: RngCore + Clone,
{
    rng: R,
    ix_max: u64,
    #[cfg(test)]
    print: u8,
}

impl<R> Shuffle<R>
where
    R: RngCore + Clone,
{
    /// Create a new [`Shuffle`].
    #[inline]
    pub const fn new(rng: R, ix_max: u64) -> Self {
        Self {
            rng,
            ix_max,
            #[cfg(test)]
            print: 0,
        }
    }

    #[cfg(test)]
    pub fn set_print(&mut self, lvl: u8) {
        self.print = lvl;

        #[cfg(test)]
        if 8 < self.print {
            eprintln!(
                "\n================================== ix_max: {}",
                fmt_u64_xd(self.ix_max)
            );
            eprintln!("  ix_max: {}", fmt_u64_xd(self.ix_max));
        }
    }

    /// The number of elements to be shuffled.
    #[inline]
    pub const fn ix_max(&self) -> u64 {
        self.ix_max
    }

    /// The number of bits required to hold an element index.
    /// Returns `1` when `ix_max` is `0`.
    #[inline]
    pub fn bits(&self) -> u8 {
        let i = self.ix_max | 1;
        let bits = u64::BITS.wrapping_sub(i.leading_zeros());
        debug_assert!(bits <= 64);
        debug_assert!(i >> bits == 0);
        debug_assert!((i << (64 - bits)) != 0);
        bits as u8
    }

    /// Indexing into the Shuffle. Returns None if index is out-of-range.
    #[inline(always)]
    pub fn get<Ix>(&self, ix: Ix) -> Option<Ix>
    where
        Ix: TryInto<u64>,
        u64: TryInto<Ix>,
    {
        ix.try_into()
            .ok()
            .and_then(|ix: u64| self.get_u64(ix))
            .and_then(|ix: u64| ix.try_into().ok())
    }

    /// Indexing into the Shuffle. Returns None if index is out-of-range.
    #[inline(never)]
    pub fn get_u64(&self, ix: u64) -> Option<u64> {
        #[cfg(test)]
        if 8 < self.print {
            eprintln!("      ix: {}", fmt_u64_xd(ix));
        }

        if self.ix_max < ix {
            return None;
        } else if self.ix_max == 0 {
            return Some(0);
        };

        let mut rng = self.rng.clone();

        if self.ix_max == 1 {
            #[cfg(test)]
            if 8 < self.print {
                eprintln!("   in ix:   {ix:01b}");
            }
            let x = self.rng.clone().next_u32() >> 31;
            let ix = ix ^ (x as u64);
            #[cfg(test)]
            if 8 < self.print {
                eprintln!("       x:   {x:01b}");
                eprintln!("  out ix:   {ix:01b}");
            }
            return Some(ix);
        }

        let bits = self.bits();
        #[cfg(test)]
        if 8 < self.print {
            eprintln!("    bits: {bits}");
        }

        debug_assert!(2 <= bits);
        debug_assert!(bits <= 64);

        let lhs_bits: u8 = 1.max(bits / 3);
        let rhs_bits: u8 = bits - lhs_bits;

        #[cfg(test)]
        if 8 < self.print {
            eprintln!("lhs_bits: {lhs_bits}");
        }
        #[cfg(test)]
        if 8 < self.print {
            eprintln!("rhs_bits: {rhs_bits}");
        }

        debug_assert_eq!(bits, lhs_bits + rhs_bits);
        debug_assert_ne!(lhs_bits, 0);
        debug_assert_ne!(rhs_bits, 0);
        debug_assert!(lhs_bits <= rhs_bits);

        const CNT_ROUNDS: usize = 4;

        let arr: [u128; CNT_ROUNDS] = std::array::from_fn(|_| {
            let mut m = rng.next_u64() as u128;
            m <<= 64;
            m |= rng.next_u64() as u128;
            m |= 1;
            m
        });

        let lhs_mask = u64::MAX >> (64 - lhs_bits);
        let rhs_mask = u64::MAX >> (64 - rhs_bits);

        #[cfg(test)]
        if 8 < self.print {
            eprintln!(
                "mask l r:   {lhs_mask:0blwid$b} {rhs_mask:0brwid$b}",
                blwid = (lhs_bits as usize),
                brwid = (rhs_bits as usize)
            );
        }

        let mut ix = ix;
        loop {
            for round_ix in 0..CNT_ROUNDS {
                #[cfg(test)]
                if 8 < self.print {
                    eprintln!("round_ix: {round_ix}");
                }

                let round_ix2 = (round_ix + 1) % CNT_ROUNDS;

                let rhs = ix & rhs_mask;
                debug_assert_eq!(rhs >> rhs_bits, 0);

                let lhs = ix >> rhs_bits;
                debug_assert_eq!(lhs >> lhs_bits, 0);

                #[cfg(test)]
                if 8 < self.print {
                    eprintln!(
                        "  l r hs:   {lhs:0blwid$b} {rhs:0brwid$b}",
                        blwid = (lhs_bits as usize),
                        brwid = (rhs_bits as usize)
                    );
                }

                let new_rhs = {
                    let mut a = rhs as u128;
                    #[cfg(test)]
                    if 8 < self.print {
                        eprintln!("      rhs:   {}", fmt_u128_x(a));
                    }

                    a = a.wrapping_add(arr[round_ix]);
                    #[cfg(test)]
                    if 8 < self.print {
                        eprintln!("        a:   {}", fmt_u128_x(a));
                    }

                    a = a.wrapping_mul(arr[round_ix2]);
                    #[cfg(test)]
                    if 8 < self.print {
                        eprintln!("        a:   {}", fmt_u128_x(a));
                    }

                    let mut new_rhs = lhs;
                    new_rhs ^= (a >> (128 - lhs_bits)) as u64;

                    new_rhs
                };
                debug_assert_eq!(new_rhs >> lhs_bits, 0);

                let new_lhs = rhs;
                debug_assert_eq!(new_lhs >> rhs_bits, 0);

                #[cfg(test)]
                if 8 < self.print {
                    eprintln!(
                        "n l r hs:   {new_lhs:0brwid$b} {new_rhs:0blwid$b}",
                        blwid = (lhs_bits as usize),
                        brwid = (rhs_bits as usize)
                    );
                }

                ix = (new_lhs << lhs_bits) | new_rhs;

                #[cfg(test)]
                if 8 < self.print {
                    eprintln!("n     ix:   {ix:0wid$b}", wid = (bits as usize));
                }

                debug_assert_eq!(ix >> bits, 0);
            }
            #[cfg(test)]
            if 8 < self.print {
                eprintln!("ix (out): {ix}");
            }

            if ix <= self.ix_max {
                break;
            }

            #[cfg(test)]
            if 8 < self.print {
                eprintln!(
                    "[relooping] ix_max: {}   ix: {}",
                    fmt_u64_xd(self.ix_max),
                    fmt_u64_xd(ix)
                );
            }
        }

        Some(ix)
    }
}

impl<R> std::fmt::Debug for Shuffle<R>
where
    R: RngCore + Clone,
{
    /// Format the value suitable for debugging output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alternate = f.alternate();
        let mut dt = f.debug_struct("Shuffle");
        dt.field("r", &"[some PRNG]");
        dt.field("ix_max", &self.ix_max);

        if alternate {
            dt.field("bits", &self.bits());
            dt.finish()
        } else {
            dt.finish_non_exhaustive()
        }
    }
}

impl<R> std::fmt::Display for Shuffle<R>
where
    R: RngCore + Clone,
{
    /// Format the value suitable for user-facing output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

//=================================================================================================|

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod t {
    use std::io::Write;

    //
    use anyhow::{Context, Result, anyhow, bail, ensure};
    use fixedbitset::FixedBitSet;
    use indoc::{indoc, writedoc};
    use insta::{assert_debug_snapshot, assert_snapshot};
    use itertools::Itertools;
    use rand_xoshiro::Xoshiro128StarStar;

    //
    use util::{
        bit_range::foreach_bit_length_range_optfirst_optlast,
        const_conv_ui128_f64::{i128_to_f64, u128_to_f64},
    };

    //
    use super::*;

    #[inline(always)]
    fn rng() -> Xoshiro128StarStar {
        rng_seed_u128(0)
    }

    #[inline(always)]
    fn rng_seed<N>(n: N) -> Xoshiro128StarStar
    where
        N: Into<u128>,
    {
        rng_seed_u128(n.into())
    }

    fn rng_seed_u128(n: u128) -> Xoshiro128StarStar {
        rng_seed_u64x2((n >> 64) as u64, n as u64)
    }

    fn rng_seed_u64x2(msb: u64, lsb: u64) -> Xoshiro128StarStar {
        // Xoshiro128StarStar gives awful results unless it's seeded well.

        const CS: [u64; 5] = [
            0x_31415926_53589793_u64,
            0x_23846264_33832795_u64,
            0x_02884197_16939937_u64,
            0x_51058209_74944592_u64,
            0x_30781640_62862089_u64,
        ];

        let ms = [msb, lsb];

        let mut it_a = CS.iter().cloned().cycle();
        let mut it_b = ms.iter().cloned().cycle();
        let mut r = 3;
        let mut seed = [0u8; 16];
        for (a, b) in zip(it_a, it_b).take(20) {
            let n = a.rotate_left(r);
            r = (r + 7) % 63;

            let n = b.rotate_left(r).wrapping_add(n);
            r = (r + 7) % 63;

            let mut ts = [0u8; 16];
            let mut rng = Xoshiro128StarStar::seed_from_u64(n);
            for _ in 0..(1 + (r % 3)) {
                rng.jump();
            }
            r = (r + 7) % 63;

            rng.fill_bytes(&mut ts);

            for (s, t) in zip(seed.iter_mut(), ts) {
                *s ^= t;
            }
        }

        Xoshiro128StarStar::from_seed(seed)
    }

    #[test]
    fn t00() {
        let s = Shuffle::new(rng(), 0);
        assert_snapshot!(s.ix_max(), @"0");
        assert_snapshot!(s.bits(), @"1");
        assert_snapshot!(s, @r#"Shuffle { r: "[some PRNG]", ix_max: 0, .. }"#);
        assert_debug_snapshot!(s, @r#"
        Shuffle {
            r: "[some PRNG]",
            ix_max: 0,
            bits: 1,
        }
        "#);
        assert_snapshot!(s.get_u64(0).unwrap(), @"0");
        assert_debug_snapshot!(s.get_u64(1), @"None");
    }

    #[test]
    fn t01a() {
        let s = Shuffle::new(rng_seed('a'), 1);
        assert_debug_snapshot!(s, @r#"
        Shuffle {
            r: "[some PRNG]",
            ix_max: 1,
            bits: 1,
        }
        "#);
        assert_snapshot!(s.get_u64(0).unwrap(), @"1");
        assert_snapshot!(s.get_u64(1).unwrap(), @"0");
        assert_debug_snapshot!(s.get_u64(2), @"None");
    }

    #[test]
    fn t01b() {
        let mut s = Shuffle::new(rng_seed('b'), 1);
        //s.set_print(255);
        assert_debug_snapshot!(s, @r#"
        Shuffle {
            r: "[some PRNG]",
            ix_max: 1,
            bits: 1,
        }
        "#);
        assert_snapshot!(s.get_u64(0).unwrap(), @"0");
        assert_snapshot!(s.get_u64(1).unwrap(), @"1");
    }

    #[test]
    fn t02a() {
        let s = Shuffle::new(rng_seed('a'), 2);
        assert_debug_snapshot!(s, @r#"
        Shuffle {
            r: "[some PRNG]",
            ix_max: 2,
            bits: 2,
        }
        "#);
        assert_snapshot!(s.get_u64(0).unwrap(), @"0");
        assert_snapshot!(s.get_u64(1).unwrap(), @"1");
        assert_snapshot!(s.get_u64(2).unwrap(), @"2");
        assert_debug_snapshot!(s.get_u64(3), @"None");
    }

    #[test]
    fn t02b() {
        let s = Shuffle::new(rng_seed('b'), 2);
        assert_snapshot!(s.get_u64(0).unwrap(), @"1");
        assert_snapshot!(s.get_u64(1).unwrap(), @"2");
        assert_snapshot!(s.get_u64(2).unwrap(), @"0");
    }

    #[test]
    fn t02c() {
        let s = Shuffle::new(rng_seed('b'), 2);
        assert_snapshot!(s.get_u64(0).unwrap(), @"1");
        assert_snapshot!(s.get_u64(1).unwrap(), @"2");
        assert_snapshot!(s.get_u64(2).unwrap(), @"0");
    }

    #[test]
    fn t03() {
        let s = Shuffle::new(rng_seed('a'), 3);
        assert_debug_snapshot!(s, @r#"
        Shuffle {
            r: "[some PRNG]",
            ix_max: 3,
            bits: 2,
        }
        "#);
        assert_snapshot!(s.get_u64(0).unwrap(), @"0");
        assert_snapshot!(s.get_u64(1).unwrap(), @"3");
        assert_snapshot!(s.get_u64(2).unwrap(), @"2");
        assert_snapshot!(s.get_u64(3).unwrap(), @"1");
        assert_debug_snapshot!(s.get_u64(4), @"None");
    }

    #[test]
    fn t04() {
        let mut s = Shuffle::new(rng(), 4);
        assert_snapshot!(s.get_u64(0).unwrap(), @"1");
        assert_snapshot!(s.get_u64(1).unwrap(), @"3");
        assert_snapshot!(s.get_u64(2).unwrap(), @"2");
        assert_snapshot!(s.get_u64(3).unwrap(), @"0");
        assert_snapshot!(s.get_u64(4).unwrap(), @"4");
    }

    #[test]
    fn t05() {
        let mut s = Shuffle::new(rng(), 5);
        assert_snapshot!(s.get_u64(0).unwrap(), @"1");
        assert_snapshot!(s.get_u64(1).unwrap(), @"5");
        assert_snapshot!(s.get_u64(2).unwrap(), @"2");
        assert_snapshot!(s.get_u64(3).unwrap(), @"0");
        assert_snapshot!(s.get_u64(4).unwrap(), @"4");
        assert_snapshot!(s.get_u64(5).unwrap(), @"3");
    }

    #[test]
    fn t15a() {
        let mut s = Shuffle::new(rng_seed('a'), 15);
        assert_snapshot!(s.get_u64(0).unwrap(), @"11");
        assert_snapshot!(s.get_u64(1).unwrap(), @"1");
        assert_snapshot!(s.get_u64(2).unwrap(), @"13");
        assert_snapshot!(s.get_u64(3).unwrap(), @"10");
        assert_snapshot!(s.get_u64(4).unwrap(), @"3");
        assert_snapshot!(s.get_u64(5).unwrap(), @"12");
        assert_snapshot!(s.get_u64(6).unwrap(), @"8");
        assert_snapshot!(s.get_u64(7).unwrap(), @"7");
        assert_snapshot!(s.get_u64(8).unwrap(), @"6");
        assert_snapshot!(s.get_u64(9).unwrap(), @"9");
        assert_snapshot!(s.get_u64(10).unwrap(), @"5");
        assert_snapshot!(s.get_u64(11).unwrap(), @"2");
        assert_snapshot!(s.get_u64(12).unwrap(), @"14");
        assert_snapshot!(s.get_u64(13).unwrap(), @"4");
        assert_snapshot!(s.get_u64(14).unwrap(), @"0");
        assert_snapshot!(s.get_u64(15).unwrap(), @"15");
        assert_debug_snapshot!(s.get_u64(16), @"None");
    }

    #[test]
    fn t15b() {
        let mut s = Shuffle::new(rng_seed('b'), 15);
        assert_snapshot!(s.get_u64(0).unwrap(), @"10");
        assert_snapshot!(s.get_u64(1).unwrap(), @"2");
        assert_snapshot!(s.get_u64(2).unwrap(), @"13");
        assert_snapshot!(s.get_u64(3).unwrap(), @"15");
        assert_snapshot!(s.get_u64(4).unwrap(), @"6");
        assert_snapshot!(s.get_u64(5).unwrap(), @"12");
        assert_snapshot!(s.get_u64(6).unwrap(), @"3");
        assert_snapshot!(s.get_u64(7).unwrap(), @"1");
        assert_snapshot!(s.get_u64(8).unwrap(), @"0");
        assert_snapshot!(s.get_u64(9).unwrap(), @"8");
        assert_snapshot!(s.get_u64(10).unwrap(), @"4");
        assert_snapshot!(s.get_u64(11).unwrap(), @"7");
        assert_snapshot!(s.get_u64(12).unwrap(), @"14");
        assert_snapshot!(s.get_u64(13).unwrap(), @"5");
        assert_snapshot!(s.get_u64(14).unwrap(), @"9");
        assert_snapshot!(s.get_u64(15).unwrap(), @"11");
        assert_debug_snapshot!(s.get_u64(16), @"None");
    }

    #[test]
    fn t_many() {
        use std::{
            io::{Cursor, Read, Write},
            // fmt::Write,
        };

        type T = u64;
        let t_bits = 64_usize;

        let bits_first: u32 = 0;
        let bits_last: u32 = 10;
        let opt_bit_length_range_first: Option<u64> = if bits_first == 0 {
            None
        } else {
            Some(1_u64 << bits_first)
        };
        let opt_bit_length_range_last: Option<u64> =
            Some(u64::MAX >> 64u32.saturating_sub(bits_last + 1));

        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::<u8>::with_capacity(8000));
        {
            let msgdest: &mut dyn Write = &mut buf;
            //let msgdest: &mut dyn Write = &mut std::io::stderr();

            foreach_bit_length_range_optfirst_optlast::<u64, _>(
                opt_bit_length_range_first,
                opt_bit_length_range_last,
                |first, last, bits| test_range(msgdest, first, last, bits),
            );

            writedoc! { msgdest, "
                ----------------------------------------------------
            " }
            .unwrap();
        }

        /*
        let s = String::from_utf8(buf.into_inner())
            .unwrap()
            .lines()
            .join("\n");
        eprintln!("{s}\n\n");
        // */
    }

    fn base10wid<N: Into<u128>>(n: N) -> usize {
        let n: u128 = n.into();
        let w = (n | 1).ilog10() as usize + 1;
        debug_assert_eq!(w, format!("{n}").len());
        w
    }

    struct Collision {
        ix_max: u64,
        rng_seed: u128,
        ix1: u64,
        ix2: u64,
        ix_out: u64,
    }

    impl Collision {
        pub fn eprintlns(&self) {
            eprintln!("Collision found:");
            eprintln!("    ix_max: {}", fmt_u64_xd(self.ix_max));
            eprintln!("    rng_seed: {}", fmt_u128_x(self.rng_seed));
            let ix_max_wid = base10wid(self.ix_max);
            eprintln!(
                "    shuffle: {} -> {}",
                fmt_u64_xdw(self.ix1, ix_max_wid),
                fmt_u64_xdw(self.ix_out, ix_max_wid)
            );
            eprintln!(
                "    and    : {} -> {}",
                fmt_u64_xdw(self.ix2, ix_max_wid),
                fmt_u64_xdw(self.ix_out, ix_max_wid)
            );
        }
    }

    fn factorial_at_least(n: u64) -> u64 {
        let mut n = n;
        let mut f = 1_u64;
        while 1 < n && f < u64::MAX {
            f = f.saturating_mul(n);
            n -= 1;
        }
        f
    }

    struct LookCollisionResults {
        ix_max: u64,
        cnt_iters: u64,
        cnt_collisions: u64,
    }

    fn look_for_collision(
        msgdest: &mut dyn Write,
        ix_max: u64,
        debug: bool,
    ) -> LookCollisionResults {
        let target_cnt_getu64_calls = {
            const MAX_TARGET_CNT_GETU64_CALLS: u64 = 500;
            let max_target_cnt_getu64_calls =
                factorial_at_least(ix_max.saturating_add(1)).saturating_mul(4);
            MAX_TARGET_CNT_GETU64_CALLS.min(max_target_cnt_getu64_calls)
        };

        let ix_max_wid = base10wid(ix_max);
        let cnt_elems = ix_max as u128 + 1;

        let cnt_iters = if ix_max == 0 {
            1_u64
        } else {
            (target_cnt_getu64_calls as u128 / cnt_elems).max(1) as u64
        };
        let iters_wid = base10wid(cnt_iters);

        if debug {
            writeln!(
                msgdest,
                "\nDoing `{cnt_iters}` iterations with ix_max: {}",
                fmt_u64_dx(ix_max)
            )
            .unwrap();
        }

        let cnt_elems_usize = usize::try_from(cnt_elems)
            .expect("Can't check that many elems, requires ix_max < usize::MAX.");
        let mut bitset = FixedBitSet::with_capacity(cnt_elems_usize);

        let mut actual_getu64_calls = 0;
        //let mut opt_collision: Option<Collision> = None;
        let mut cnt_collisions = 0_u64;
        'iter: for iter in 1_u64..=cnt_iters {
            bitset.clear();

            if debug {
                writeln!(msgdest,
                    "[{iter:iters_wid$}/{cnt_iters:iters_wid$}] -------------------------- ix_max: {} -------- iter {iter} --------------------------",
                    fmt_u64_xd(ix_max)
                ).unwrap();
                writeln!(
                    msgdest,
                    "[{iter:iters_wid$}/{cnt_iters:iters_wid$}] ix_max: {}",
                    fmt_u64_xdw(ix_max, 4)
                )
                .unwrap();
            }

            let rng_seed = rng_seed_u64x2(ix_max, iter).random::<u128>();

            if debug {
                writeln!(
                    msgdest,
                    "[{iter:iters_wid$}/{cnt_iters:iters_wid$}] Rng seed: {}",
                    fmt_u128_x(rng_seed)
                )
                .unwrap();
            }

            let shuffle = {
                let rng = rng_seed_u128(rng_seed);
                let mut shuffle = Shuffle::new(rng, ix_max);
                //if debug { shuffle.set_print(255) }
                shuffle
            };

            for ix2 in 0..=ix_max {
                let ix_out = shuffle.get_u64(ix2).unwrap();
                /*
                let ix_out = match std::panic::catch_unwind(|| shuffle.get_u64(ix2)) {
                    Ok(Some(ix_out)) => ix_out,
                    _ => {
                        let s = indoc::formatdoc! { "
                            Error: panic in `Shuffle::get_u64()`:
                            shuffle: {shuffle:?}
                            Rng seed: {rng_seed}
                            ix2: {ix2}
                        ", rng_seed=fmt_u128_x(rng_seed) };
                        eprintln!("{s}");
                        writeln!(msgdest, "{s}").unwrap();
                        /*
                        let mut shuffle = Shuffle::new(Hasher_from_seed_u64(iter), ix_max);
                        shuffle.set_print(255);
                        std::panic::catch_unwind(|| {
                            shuffle.get_u64(ix2)
                        });
                        // */
                        cnt_collisions += 1;
                        continue 'iter;
                    }
                };
                // */

                actual_getu64_calls += 1;

                if debug {
                    writeln!(msgdest, "[{iter:iters_wid$}/{cnt_iters:iters_wid$}] [{ix2:ix_max_wid$}..={cnt_elems}] shuffle: {ix2} -> {ix_out}",
                    ix2=fmt_u64_xdw(ix2, ix_max_wid), ix_out=fmt_u64_xdw(ix_out, ix_max_wid)).unwrap();
                }

                let prev_bit = bitset.put(usize::try_from(ix_out).unwrap());

                if prev_bit {
                    // Collision found
                    let mut opt_ix1: Option<u64> = None;
                    for ix1 in 0..ix2 {
                        if shuffle.get_u64(ix1).unwrap() == ix_out {
                            opt_ix1 = Some(ix1);
                            break;
                        }
                    }

                    if let Some(ix1) = opt_ix1 {
                        let collision_ = Collision {
                            ix_max,
                            rng_seed,
                            ix1,
                            ix2,
                            ix_out,
                        };
                    } else {
                        writedoc!{ msgdest, "
                            [{iter:iters_wid$}/{cnt_iters:iters_wid$}] [{ix2:ix_max_wid$}..={cnt_elems}] shuffle: {ix2} -> {ix_out}
                            Error: Can't find ix1 -> {ix_out} !
                        ", ix2=fmt_u64_xdw(ix2, ix_max_wid), ix_out=fmt_u64_xdw(ix_out, ix_max_wid)
                        }.unwrap();
                    };

                    cnt_collisions += 1;
                    continue 'iter;
                }
            }

            if debug {
                writeln!(
                    msgdest,
                    "[{iter:iters_wid$}/{cnt_iters:iters_wid$}] No collision."
                )
                .unwrap();
            }

            let cnt_ones = bitset.count_ones(..) as u128;

            if debug || cnt_ones != cnt_elems {
                writeln!(msgdest, "[{iter:iters_wid$}/{cnt_iters:iters_wid$}] Image elements identified: {cnt_ones}").unwrap();
                writeln!(msgdest, "[{iter:iters_wid$}/{cnt_iters:iters_wid$}] Image elements expected  : {cnt_elems}").unwrap();
                assert_eq!(cnt_ones, cnt_elems);
            }
        }

        /*
        #[cfg(debug_assertions)]
        if nt_collisions == 0 {
            let target = i128_to_f64(target_cnt_getu64_calls.max(1).into());
            let actual = i128_to_f64(actual_getu64_calls);
            let diff = actual - target;
            let pct = diff / target * 100.0;
            let abs_diff = diff.abs();
            if ix_max != 0 {
                let pct_of_cnt_elems = abs_diff / u128_to_f64(cnt_elems) * 100.0;
                if 50.1 < pct_of_cnt_elems {
                    writeln!(msgdest, "actual `get_u64()` calls: {actual}").unwrap();
                    writeln!(msgdest, "target was              : {target}").unwrap();
                    writeln!(msgdest, "difference              : {diff}").unwrap();
                    writeln!(msgdest, "                        : {pct:.1} % off target").unwrap();
                    writeln!(
                        msgdest,
                        "                        : {pct_of_cnt_elems:.1} % of cnt_elems"
                    )
                    .unwrap();
                }
            }
        }
        // */

        LookCollisionResults {
            ix_max,
            cnt_iters,
            cnt_collisions,
        }
    }

    fn test_ix_max(ix_max: u64) {
        let msgdest = &mut std::io::stderr();

        let LookCollisionResults {
            ix_max,
            cnt_iters,
            cnt_collisions,
        } = look_for_collision(msgdest, ix_max, false);

        if 0 < cnt_iters && cnt_collisions == 0 {
            #[allow(clippy::needless_return)] //? TODO: Remove temp development code
            return;
        } else {
            //? collision.eprintlns();
            panic!("Collision(s) detected.");
        }
    }

    struct FmtWidths {
        first: u64,
        last: u64,
        bits: u32,
        hexs: usize,
        bwid: usize,
        bs: String,
        hwid: usize,
        hs: String,
        dec_range_wid: usize,
        dec_range_s: String,
    }

    impl FmtWidths {
        pub fn new(first: u64, last: u64, bits: u32) -> Self {
            let bwid = bits.next_multiple_of(4).max(6) as usize;
            let bs = format!("{:-^bwid$}", "bits");

            let hexs = bits.div_ceil(4) as usize;
            let hwid = hexs.next_multiple_of(4).max(5);

            let hs = format!("{:-^hwid$}", "hex");

            let dec_range_wid = 5;
            let dec_range_s = format!("{:-^dec_range_wid$}", "dec");

            Self {
                first,
                last,
                bits,
                hexs,
                bwid,
                bs,
                hwid,
                hs,
                dec_range_wid,
                dec_range_s,
            }
        }

        pub fn format_bits(&self, u: u64) -> String {
            format!("{u:0bits$b}", bits = self.bits as usize)
        }

        pub fn format_hexs(&self, u: u64) -> String {
            format!("{u:0hexs$X}", hexs = self.hexs)
        }

        pub fn write_range_header(&self, msgdest: &mut dyn Write) {
            let dec_range = format!("{} ..= {}", self.first, self.last);
            let bf = self.format_bits(self.first);
            let bl = self.format_bits(self.last);
            let hf = self.format_hexs(self.first);
            let hl = self.format_hexs(self.last);

            writedoc! { msgdest, "
                ----------------------------------------------------
                     {bs:^bwid$}  {hs:^hwid$}    {dec_range_s:>dec_range_wid$}
                     {bf:>bwid$}  {hf:>hwid$}    {first:>dec_range_wid$}
                ..=  {bl:>bwid$}  {hl:>hwid$}    {last:>dec_range_wid$}
            ",
            bs=self.bs, bwid=self.bwid,
            hs=self.hs, hwid=self.hwid,
            dec_range_wid=self.dec_range_wid,
            dec_range_s=self.dec_range_s,
            first=self.first,
            last=self.last,
            }
            .unwrap();
        }

        pub fn write_test_max_n(
            &self,
            msgdest: &mut dyn Write,
            max_n: u64,
            cnt_collisions: u64,
            cnt_iters: u64,
        ) {
            let bwid = self.bwid;
            let hwid = self.hwid;
            let bx = self.format_bits(max_n);
            let hx = self.format_hexs(max_n);
            let dx = max_n;
            let dec_range_wid = self.dec_range_wid;
            write!(
                msgdest,
                "     {bx:>bwid$}  {hx:>hwid$}    {dx:>dec_range_wid$}"
            )
            .unwrap();

            if 0 < cnt_collisions {
                let cnt_iters = (cnt_iters as f64).max(1.0e-9);
                let pct = (cnt_collisions as f64) / cnt_iters * 100.0;
                writeln!(
                    msgdest,
                    "     {cnt_collisions} collisions detected in {cnt_iters} seeds ({pct:.1}%)"
                )
                .unwrap();
            } else {
                writeln!(msgdest, "     ok").unwrap();
            }
        }
    }

    fn test_range(msgdest: &mut dyn Write, first: u64, last: u64, bits: u32) {
        let fmt_widths = FmtWidths::new(first, last, bits);
        let fmt_widths = &fmt_widths;
        fmt_widths.write_range_header(msgdest);

        let mut cnt_collisions = 0_u64;
        for max_n in first..=last {
            cnt_collisions += test_maxn(fmt_widths, msgdest, max_n);
        }
    }

    fn test_maxn(fmt_widths: &FmtWidths, msgdest: &mut dyn Write, ix_max: u64) -> u64 {
        let LookCollisionResults {
            ix_max,
            cnt_iters,
            cnt_collisions,
        } = look_for_collision(msgdest, ix_max, false);

        fmt_widths.write_test_max_n(msgdest, ix_max, cnt_collisions, cnt_iters);

        cnt_collisions
    }
}
