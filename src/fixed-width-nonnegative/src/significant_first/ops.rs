// Copyright (C) Microsoft Corporation. All rights reserved.

//#![cfg_attr(rustfmt, rustfmt_skip)]
#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(elided_lifetimes_in_paths)]
#![allow(clippy::assertions_on_constants)]
//#![allow(clippy::type_complexity)]
//#![allow(clippy::empty_line_after_doc_comments)] //? TODO: Remove temp development code
//#![allow(clippy::let_and_return)] //? TODO: Remove temp development code
//#![allow(clippy::needless_lifetimes)] //? TODO: Remove temp development code
//#![allow(dead_code)] //? TODO: Remove temp development code
//#![allow(unused_assignments)] //? TODO: Remove temp development code
//#![allow(unused_braces)] //? TODO: Remove temp development code
//#![allow(unused_imports)] //? TODO: Remove temp development code
//#![allow(unused_mut)] //? TODO: Remove temp development code
//#![allow(unused_variables)] //? TODO: Remove temp development code
//#![allow(unreachable_code)] //? TODO: Remove temp development code
//#![allow(non_camel_case_types)] //? TODO: Remove temp development code
//#![allow(non_snake_case)] //? TODO: Remove temp development code
//#![allow(non_upper_case_globals)] //? TODO: Remove temp development code
//#![allow(noop_method_call)] //? TODO: Remove temp development code

use crate::primitive_unsigned::*;

use super::{AsLsfSlicePrimitiveUnsigned, LeastSignificantFirstPrimitiveUnsignedArray};

//=================================================================================================|

pub fn add_returning_carry<Elem, const N: usize, Lhs, Rhs, L>(
    lhs: &Lhs,
    rhs: &Rhs,
) -> (LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>, u8)
where
    Lhs: AsLsfSlicePrimitiveUnsigned,
    &'static <Lhs as AsLsfSlicePrimitiveUnsigned>::Elem: std::ops::Deref<Target = Elem>,
    Rhs: AsLsfSlicePrimitiveUnsigned,
    &'static <Rhs as AsLsfSlicePrimitiveUnsigned>::Elem: std::ops::Deref<Target = Elem>,
    Elem: PrimitiveUnsignedAtLeast8 + NextLargerThan, // need room to carry
    Elem: std::convert::Into<L>,
    Elem: Trunc<u8>,
    L: PrimitiveUnsigned,
    L: Trunc<u8>,
    L: Trunc<Elem>,
    <Lhs as AsLsfSlicePrimitiveUnsigned>::Elem: std::convert::Into<L>,
    <Rhs as AsLsfSlicePrimitiveUnsigned>::Elem: std::convert::Into<L>,
    &'static L: std::ops::Deref<Target = <Elem as NextLargerThan>::Type>,
{
    if N == 0 {
        return (Default::default(), 0);
    }

    let mut lhs_iter = lhs.as_lsf_slice().iter();
    let mut rhs_iter = rhs.as_lsf_slice().iter();

    let mut c = 0_u8;
    let r = LeastSignificantFirstPrimitiveUnsignedArray::from_fn(|_i| {
        debug_assert!(_i < N);

        // Unwrap() is justified here because `N != 0`
        #[allow(clippy::unwrap_used)]
        let a: L = (*lhs_iter.next().unwrap()).into();

        // Unwrap() is justified here because `N != 0`
        #[allow(clippy::unwrap_used)]
        let mut a: L = a + (*rhs_iter.next().unwrap()).into();

        a += std::convert::Into::<L>::into(c);

        let b = a >> Elem::BITS;
        debug_assert!(b <= L::ONE);

        c = Trunc::<u8>::trunc(b);

        if N <= _i + 1 {
            debug_assert!(lhs_iter.next().is_none());
            debug_assert!(rhs_iter.next().is_none());
        }

        Trunc::<Elem>::trunc(a)
    });

    (r, c)
}
