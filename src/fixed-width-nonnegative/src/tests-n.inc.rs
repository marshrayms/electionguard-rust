// Copyright (C) Microsoft Corporation. All rights reserved.

// NOTE: This file is intended to be include!()ed, in a specific context and not built directly.

use static_assertions::{
    assert_impl_all,
    const_assert,
    const_assert_eq,
};

use super::*;

const TEST_BYTES: usize = TEST_BITS/8;
const TEST_HEXDG: usize = TEST_BYTES*2;

#[test]
fn nn_consts() {
    // Gratuitous overflow checks.
    const_assert_eq!(TEST_BYTES*8, TEST_BITS);
    const_assert_eq!(TEST_HEXDG*4, TEST_BITS);

    // Verify `BITS`.
    const_assert_eq!(Nonnegative::BITS as u128, TEST_BITS as u128);

    //// Verify that `UNDERLYING_IMPL_CRATE_NAME` is a valid identifier.
    assert!(!Nonnegative::UNDERLYING_IMPL_CRATE_NAME.is_empty());
    let module_name = Nonnegative::UNDERLYING_IMPL_CRATE_NAME.replace('-', "_");
    assert!(syn::parse_str::<syn::Ident>(module_name.as_str()).is_ok());
    assert!(Nonnegative::UNDERLYING_IMPL_CRATE_NAME != "_");
}

#[test]
fn std_fmt_upperlowerhex_zero() {
    let nn_zero = Nonnegative::zero();

    let mut hex_zero = "0".repeat(TEST_BITS/4);
    assert_eq!(format!("{nn_zero:X}"), hex_zero);
    assert_eq!(format!("{nn_zero:x}"), hex_zero);

    hex_zero.insert_str(0, "0x");
    assert_eq!(format!("{nn_zero:#X}"), hex_zero);
    assert_eq!(format!("{nn_zero:#x}"), hex_zero);
}

#[test]
fn std_fmt_upperlowerhex_one() {
    let nn_one = Nonnegative::one();

    let mut hex_one = "0".repeat(TEST_BITS/4 - 1);
    hex_one.push('1');
    assert_eq!(format!("{nn_one:X}"), hex_one);
    assert_eq!(format!("{nn_one:x}"), hex_one);

    hex_one.insert_str(0, "0x");
    assert_eq!(format!("{nn_one:#X}"), hex_one);
    assert_eq!(format!("{nn_one:#x}"), hex_one);
}

#[test]
fn std_fmt_upperlowerhex_allones() {
    let nn = Nonnegative::all_ones();
    assert_eq!(format!("{:X}", nn), "F".repeat(TEST_HEXDG));
    assert_eq!(format!("{:x}", nn), "f".repeat(TEST_HEXDG));
}

fn hx(nn: &Nonnegative)->String {
    format!("{nn:X}")
}

#[test]
fn from_le_bytes_arr() {
    let mut aby = [ 0_u8; TEST_BYTES ];
    assert_eq!(hx(&Nonnegative::from_le_bytes_arr(aby)), "0".repeat(TEST_HEXDG));
    aby[TEST_BYTES - 1] = 1;
    assert_eq!(hx(&Nonnegative::from_le_bytes_arr(aby)), "01".to_owned()+&"0".repeat(TEST_HEXDG-2));
}

#[test]
fn from_be_bytes_arr() {
    let mut aby = [ 0_u8; TEST_BYTES ];
    assert_eq!(hx(&Nonnegative::from_be_bytes_arr(aby)), "0".repeat(TEST_HEXDG));
    aby[TEST_BYTES - 1] = 1;
    assert_eq!(hx(&Nonnegative::from_be_bytes_arr(aby)), "0".repeat(TEST_HEXDG-1)+"1");
}

#[test]
fn to_le_bytes_arr() {
    let mut aby = [0_u8; TEST_BYTES];
    let aby2: [u8; TEST_BYTES] = Nonnegative::from_le_bytes_arr(aby).to_le_bytes_arr();
    assert_eq!(aby2, aby);

    aby[0] = 1;
    let aby2: [u8; TEST_BYTES] = Nonnegative::from_le_bytes_arr(aby).to_le_bytes_arr();
    assert_eq!(aby2, aby);

    aby[3] = 0x47;
    let aby2: [u8; TEST_BYTES] = Nonnegative::from_le_bytes_arr(aby).to_le_bytes_arr();
    assert_eq!(aby2, aby);
}

#[test]
fn to_be_bytes_arr() {
    let mut aby = [0_u8; TEST_BYTES];
    let aby2: [u8; TEST_BYTES] = Nonnegative::from_be_bytes_arr(aby).to_be_bytes_arr();
    assert_eq!(aby2, aby);

    aby[TEST_BYTES - 1] = 1;
    let aby2: [u8; TEST_BYTES] = Nonnegative::from_be_bytes_arr(aby).to_be_bytes_arr();
    assert_eq!(aby2, aby);

    aby[3] = 0x47;
    let aby2: [u8; TEST_BYTES] = Nonnegative::from_be_bytes_arr(aby).to_be_bytes_arr();
    assert_eq!(aby2, aby);
}

#[test]
fn stdconvertfrom_u8() {
    for by in [
        0_u8, 1, 2, 3, 127, 128, 253, 254, 255
    ] {
        let nn = <Nonnegative as std::convert::From<u8>>::from(by);
        assert_eq!(format!("{nn:X}"), format!("{by:0wid$X}", wid=TEST_HEXDG));
    }
}

#[test]
fn stdconvertfrom_u16() {
    let range = 0_u16 .. (1u16 << 2);
    for a in 0..4_u16 {
        for b in -2..=2_i32 {
            let Some(ab) = u16::try_from(a.reverse_bits() as i32 + b).ok() else { continue; };
            let nn = <Nonnegative as std::convert::From<u16>>::from(ab);
            assert_eq!(format!("{nn:X}"), format!("{ab:0wid$X}", wid=TEST_HEXDG));
        }
    }
}

#[test]
fn stdconvertfrom_u32() {
    let range = 0_u32 .. (1u32 << 2);
    for a in 0..4_u32 {
        for b in -2..=2_i64 {
            let Some(ab) = u32::try_from(a.reverse_bits() as i64 + b).ok() else { continue; };
            let nn = <Nonnegative as std::convert::From<u32>>::from(ab);
            assert_eq!(format!("{nn:X}"), format!("{ab:0wid$X}", wid=TEST_HEXDG));
        }
    }
}

#[test]
fn stdconvertfrom_u64() {
    let range = 0_u64 .. (1u64 << 2);
    for a in 0..4_u64 {
        for b in -2..=2_i128 {
            let Some(ab) = u64::try_from(a.reverse_bits() as i128 + b).ok() else { continue; };
            let nn = <Nonnegative as std::convert::From<u64>>::from(ab);
            assert_eq!(format!("{nn:X}"), format!("{ab:0wid$X}", wid=TEST_HEXDG));
        }
    }
}

/*
#[test]
fn stdconvertfrom_u128() {
    //? TODO
    let range = 0_u64 .. (1u64 << 2);
    for a in 0..4_u64 {
        for b in -2..=2_i128 {
            let Some(ab) = u64::try_from(a.reverse_bits() as i128 + b).ok() else { continue; };
            let nn = <Nonnegative as std::convert::From<u64>>::from(ab);
            assert_eq!(format!("{nn:X}"), format!("{ab:0wid$X}", wid=TEST_HEXDG));
        }
    }
}
*/

fn wrapping_add_self_print() {
    if TEST_BITS == 256 {
        eprintln!("\n\n============================== {}\n", TEST_NUMIPML.crate_name());

        let a_nn = [
            Nonnegative::zero(),
            Nonnegative::one(),
            Nonnegative::all_ones(),
        ];
        let a_name = [ "0", "1", "MAX" ];

        let alen = a_nn.len();
        for val_ix_pr in 0..(alen*alen) {
            let lhs_ix = val_ix_pr / alen;
            let rhs_ix = val_ix_pr % alen;

            let lhs = a_nn[lhs_ix].clone();
            let rhs = a_nn[rhs_ix].clone();

            eprintln!("wrapping_add: {} + {}", a_name[lhs_ix], a_name[rhs_ix]);
            eprintln!("    lhs: {lhs:0wid$X}", wid=TEST_HEXDG);
            eprintln!("    rhs: {rhs:0wid$X}", wid=TEST_HEXDG);
            let res = lhs.wrapping_add(&rhs);
            eprintln!("    res: {res:0wid$X}", wid=TEST_HEXDG);
            eprintln!();
        }
        eprintln!();
    }
}

#[test]
fn wrapping_add_self() {
    //wrapping_add_self_print();
    assert_eq!(format!("{:X}", Nonnegative::zero().wrapping_add(&Nonnegative::zero())), format!("{:0wid$X}", 0, wid=TEST_HEXDG));
    assert_eq!(format!("{:X}", Nonnegative::zero().wrapping_add(&Nonnegative::one())), format!("{:0wid$X}", 1, wid=TEST_HEXDG));
    assert_eq!(format!("{:X}", Nonnegative::zero().wrapping_add(&Nonnegative::all_ones())), "F".repeat(TEST_HEXDG));
    assert_eq!(format!("{:X}", Nonnegative::one().wrapping_add(&Nonnegative::zero())), format!("{:0wid$X}", 1, wid=TEST_HEXDG));
    assert_eq!(format!("{:X}", Nonnegative::one().wrapping_add(&Nonnegative::one())), format!("{:0wid$X}", 2, wid=TEST_HEXDG));
    assert_eq!(format!("{:X}", Nonnegative::all_ones().wrapping_add(&Nonnegative::zero())), "F".repeat(TEST_HEXDG));
    assert_eq!(format!("{:X}", Nonnegative::all_ones().wrapping_add(&Nonnegative::one())),  "0".repeat(TEST_HEXDG));
    assert_eq!(format!("{:X}", Nonnegative::all_ones().wrapping_add(&Nonnegative::all_ones())), "F".repeat(TEST_HEXDG-1)+"E");
    assert_eq!(format!("{:X}", Nonnegative::one().wrapping_add(&Nonnegative::all_ones())), format!("{:0wid$X}", 0, wid=TEST_HEXDG));
}

/*
assert_impl_all!(Nonnegative: FwnnOps);

//assert_impl_all!(Nonnegative: PartialEq);
//assert_impl_all!(Nonnegative: Eq);
//assert_impl_all!(Nonnegative: PartialEq, Eq);

//assert_impl_all!(Nonnegative: PartialOrd);
//assert_impl_all!(Nonnegative: Ord);

//assert_impl_all!(Nonnegative: std::ops::Add<Nonnegative, Output = Nonnegative>);
//assert_impl_all!(Nonnegative: num_traits::identities::Zero); // Sized + Add<Self>
//?TODO assert_impl_all!(Nonnegative: num_traits::identities::One); // Sized + Mul<Self>
//?TODO assert_impl_all!(Nonnegative: std::ops::Sub<Nonnegative, Output = Nonnegative>);
//?TODO assert_impl_all!(Nonnegative: std::ops::Mul<Nonnegative, Output = Nonnegative>);
//?TODO assert_impl_all!(Nonnegative: std::ops::Div<Nonnegative, Output = Nonnegative>);
//?TODO assert_impl_all!(Nonnegative: std::ops::Rem<Nonnegative, Output = Nonnegative>);
//?TODO assert_impl_all!(Nonnegative: num_traits::NumOps); // Add + Sub + Mul + Div + Rem
//?TODO assert_impl_all!(Nonnegative: num_traits::Num); // PartialEq + Zero + One + NumOps
//?TODO assert_impl_all!(Nonnegative: num_integer::Integer); // Sized + Num + PartialOrd + Ord + Eq

//? TODO consider assert_impl_all!(Nonnegative: std::ops::AddAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::BitAnd);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::BitAndAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::BitOr);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::BitOrAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::BitXor);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::BitXorAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::Div);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::DivAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::Mul);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::MulAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::Not);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::Rem);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::RemAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::Shl);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::ShlAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::Shr);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::ShrAssign);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::Sub);
//? TODO consider assert_impl_all!(Nonnegative: std::ops::SubAssign);

//? TODO consider std::convert::From ?
//? TODO consider std::convert::Into ?


//? TODO more per-backend-bits tests
*/
