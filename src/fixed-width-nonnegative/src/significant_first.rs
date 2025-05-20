// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::manual_assert)]
#![allow(clippy::assertions_on_constants)]
//-
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)] //? TODO: Remove temp development code
#![allow(non_snake_case)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code

pub mod ops;

use std::{
    iter::repeat,
    mem::{size_of, size_of_val},
};

use anyhow::{Context, Result, anyhow, bail, ensure};
//use static_assertions::const_assert_eq;
use paste::paste;

use cfg_if::cfg_if;

#[rustfmt::skip]
use crate::{
    primitive_unsigned::*
};

//=================================================================================================|

/// Trait for types that can be represented as a least-significant-first slice of `Elem`,
/// which is a primitive unsigned type (at least `u8` width).
pub trait AsLsfSlicePrimitiveUnsigned {
    type Elem: PrimitiveUnsigned;

    /// Returns a slice of elements covering the entire least-significant-first primitive unsigned array.
    fn as_lsf_slice(&self) -> &[Self::Elem];

    /// Returns a mutable slice covering the entire least-significant-first primitive unsigned array.
    fn as_lsf_mut_slice(&mut self) -> &mut [Self::Elem];
}

pub trait AsLsfSlicePrimitiveUnsignedExt: AsLsfSlicePrimitiveUnsigned {
    /// Returns whether self is equal to a slice that the caller asserts is in LSF order.
    /// Although the slice lengths can differ, the `Elem` types must be the same.
    ///
    /// Prefer to use `eq_vartime` when possible.
    #[allow(clippy::needless_range_loop)] //? TODO clippy
    fn eq_vartime_slice_caller_promises_is_lsf(&self, that_slice: &[Self::Elem]) -> bool {
        let self_slice: &[Self::Elem] = self.as_lsf_slice();
        let self_len: usize = self_slice.len();
        let that_len: usize = that_slice.len();
        let common_len: usize = self_len.min(that_len);

        for ix in 0..common_len {
            if self_slice[ix] != that_slice[ix] {
                return false;
            }
        }

        for ix in common_len..self_len {
            if self_slice[ix] != Self::Elem::ZERO {
                return false;
            }
        }

        for ix in common_len..that_len {
            if that_slice[ix] != Self::Elem::ZERO {
                return false;
            }
        }
        true
    }

    /// Returns whether the two are equal by comparing slices. Although the length can differ,
    /// the `Elem` types must be the same.
    fn eq_vartime<T>(&self, that: &T) -> bool
    where
        T: AsLsfSlicePrimitiveUnsigned<Elem = Self::Elem> + ?Sized,
    {
        let that_slice: &[Self::Elem] = that.as_lsf_slice();
        self.eq_vartime_slice_caller_promises_is_lsf(that_slice)
    }
    /*
    fn eq_vartime<L, T>(&self, that: T) -> bool
    where
        L: AsLsfSlicePrimitiveUnsigned<Elem = Self::Elem> + ?Sized,
        T: ::std::convert::AsRef<L>,
    {
        let that_slice: &[Self::Elem] = that.as_ref().as_lsf_slice();
        self.eq_vartime_slice_caller_promises_is_lsf(that_slice)
    }
    // */

    fn as_lsf_iter_u8(&self) -> impl Iterator<Item = u8> {
        <Self as AsLsfSlicePrimitiveUnsigned>::Elem::slice_self_to_le_iter_u8(self.as_lsf_slice())
    }
}

impl<T> AsLsfSlicePrimitiveUnsignedExt for T where T: AsLsfSlicePrimitiveUnsigned {}

//=================================================================================================|

/// Trait for types that can be represented as a least-significant-first slice of `u8`.
pub trait AsLsfSlice_u8 {
    /// Returns a slice of least-significant-first `u8`.
    fn as_lsf_slice_u8(&self) -> &[u8];

    /// Returns a mutable slice of least-significant-first `u8`.
    fn as_lsf_mut_slice_u8(&mut self) -> &mut [u8];
}

//=================================================================================================|

/// Trait for types that can be represented as a least-significant-first slice of `u16`.
pub trait AsLsfSlice_u16 {
    /// Returns a slice of least-significant-first `u16`.
    fn as_lsf_slice_u16(&self) -> &[u16];

    /// Returns a mutable slice of least-significant-first `u16`.
    fn as_lsf_mut_slice_u16(&mut self) -> &mut [u16];
}

//=================================================================================================|

/// Trait for types that can be represented as a least-significant-first slice of `u32`.
pub trait AsLsfSlice_u32 {
    /// Returns a slice of least-significant-first `u32`.
    fn as_lsf_slice_u32(&self) -> &[u32];

    /// Returns a mutable slice of least-significant-first `u32`.
    fn as_lsf_mut_slice_u32(&mut self) -> &mut [u32];
}

//=================================================================================================|

/// Trait for types that can be represented as a least-significant-first slice of `u64`.
pub trait AsLsfSlice_u64 {
    /// Returns a slice of least-significant-first `u64`.
    fn as_lsf_slice_u64(&self) -> &[u64];

    /// Returns a mutable slice of least-significant-first `u64`.
    fn as_lsf_mut_slice_u64(&mut self) -> &mut [u64];
}

//=================================================================================================|

/// Trait for types that can be represented as a least-significant-first slice of `u128`.
pub trait AsLsfSlice_u128 {
    /// Returns a slice of least-significant-first `u128`.
    fn as_lsf_slice_u128(&self) -> &[u128];

    /// Returns a mutable slice of least-significant-first `u128`.
    fn as_lsf_mut_slice_u128(&mut self) -> &mut [u128];
}

//=================================================================================================|

//? #[derive(Copy, bytemuck::Pod)] // Pod requires Copy for some unknown reason
#[derive(Clone, bytemuck::Zeroable, zeroize::ZeroizeOnDrop)]
#[repr(transparent)]
pub struct LeastSignificantFirstPrimitiveUnsignedArray<Elem, const N: usize>([Elem; N])
where
    Elem: PrimitiveUnsigned + bytemuck::Zeroable,
    [Elem; N]: bytemuck::Zeroable;

/*
the trait `std::marker::Copy` is not implemented for `significant_first::LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>`
note: required by a bound in `zeroize::DefaultIsZeroes`

impl<Elem, const N: usize> zeroize::DefaultIsZeroes
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
}
*/

// Derive macro `bytemuck::TransparentWrapper` didn't work because of constraint.
unsafe impl<Elem, const N: usize> bytemuck::TransparentWrapper<[Elem; N]>
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
}

impl<Elem, const N: usize> LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
    //<Elem as PrimitiveUnsigned>::Type_array_u8: [u8; _],
{
    pub const N: usize = N;
    pub const BYTES: usize = N * Elem::SIZE;
    pub const BITS: usize = Self::BYTES * 8;

    pub const fn elem_size_in_bytes(&self) -> usize {
        Elem::SIZE
    }

    pub const fn elem_size_in_bits(&self) -> usize {
        Elem::SIZE * 8
    }

    pub const fn array_size_in_bytes(&self) -> usize {
        Self::BYTES
    }

    pub const fn array_size_in_bits(&self) -> usize {
        Self::BITS
    }

    pub const ZERO: Self = Self([Elem::ZERO; N]);

    pub const ONE: Self = {
        let mut self_ = Self::ZERO;
        self_.0[0] = Elem::ONE;
        self_
    };

    pub const MAX: Self = Self([Elem::MAX; N]);

    /// Compare to: `std::array::from_fn`.
    #[inline]
    pub fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> Elem,
    {
        Self(std::array::from_fn(f))
    }

    #[inline]
    pub fn try_from_le_bytes_arr<const M: usize>(src: [u8; M]) -> Result<Self> {
        Self::try_from_le_slice_u8(&src)
    }

    #[inline]
    pub fn try_from_be_bytes_arr<const M: usize>(src: [u8; M]) -> Result<Self> {
        cfg_if! {
            if #[cfg(target_endian = "little")] {
                let mut src = src;
                src.reverse();
                //? Self::try_from_le_bytes_arr(src)
                Self::try_from_le_slice_u8(&src)
            } else {
                ensure!(size_of::<[u8; M]>() == size_of::<[Elem; N]>());
                let mut a: [Elem; N] = std::mem::transmute(src);
                a.reverse();
                Ok(Self(a))
            }
        }
    }

    #[inline]
    pub fn try_from_lsf_iter<I: Iterator<Item = Elem>>(src: I) -> Result<Self> {
        let mut iter = src.fuse();
        let a = std::array::from_fn(|_i| iter.next().unwrap_or_default());
        for elem in iter {
            if elem != Elem::ZERO {
                bail!("Src value too large");
            }
        }
        Ok(Self(a))
    }

    #[inline]
    pub fn try_from_le_iter_u8<I: Iterator<Item = u8>>(src: I) -> Result<Self> {
        let mut iter = Elem::le_iter_u8_to_iter_self(src.fuse());
        let a = std::array::from_fn(|_i| iter.next().unwrap_or_default());
        for elem in iter {
            if elem != Elem::ZERO {
                bail!("Src value too large");
            }
        }
        Ok(Self(a))
    }

    #[inline]
    pub fn try_from_le_slice_u8(src: &[u8]) -> Result<Self> {
        Self::try_from_le_iter_u8(src.iter().copied())
    }

    #[inline]
    pub const fn as_lsf_array(&self) -> &[Elem; N] {
        &self.0
    }

    #[inline]
    pub fn as_mut_lsf_array(&mut self) -> &mut [Elem; N] {
        &mut self.0
    }
}

impl<Elem, const N: usize> LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
    Self: AsLsfSlice_u8,
{
    #[inline]
    pub fn try_from_be_bytes_slice(src: &[u8]) -> Result<Self> {
        let mut self_ = Self::ZERO;

        let mut dst = AsLsfSlice_u8::as_lsf_mut_slice_u8(&mut self_);
        ensure!(src.len() == dst.len(), "`Src` must be same size");
        dst.copy_from_slice(src);

        #[cfg(target_endian = "little")]
        dst.reverse();

        Ok(self_)
    }
}

impl<Elem, const N: usize> std::default::Default
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
    fn default() -> Self {
        Self::ZERO
    }
}

impl<Elem, const N: usize> std::convert::AsRef<[Elem; N]>
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
    #[inline]
    fn as_ref(&self) -> &[Elem; N] {
        &self.0
    }
}

impl<Elem, const N: usize> std::convert::AsMut<[Elem; N]>
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [Elem; N] {
        &mut self.0
    }
}

impl<Elem, const N: usize> AsLsfSlicePrimitiveUnsigned
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
    type Elem = Elem;

    fn as_lsf_slice(&self) -> &[Self::Elem] {
        self.as_ref().as_slice()
    }

    fn as_lsf_mut_slice(&mut self) -> &mut [Self::Elem] {
        self.as_mut().as_mut_slice()
    }
}

//=================================================================================================|

macro_rules! impl_AsLsfSlicePrimitiveUnsigned {
    ($pu:ident) => {
        paste! {
            impl<const N: usize> [<AsLsfSlice_ $pu>] for LeastSignificantFirstPrimitiveUnsignedArray<$pu, N> {
                fn [<as_lsf_slice_ $pu>](&self) -> &[$pu] { self.0.as_slice() }
                fn [<as_lsf_mut_slice_ $pu>](&mut self) -> &mut [$pu] { self.0.as_mut_slice() }
            }
        }
    };
}
impl_AsLsfSlicePrimitiveUnsigned!(u8);
impl_AsLsfSlicePrimitiveUnsigned!(u16);
impl_AsLsfSlicePrimitiveUnsigned!(u32);
impl_AsLsfSlicePrimitiveUnsigned!(u64);
impl_AsLsfSlicePrimitiveUnsigned!(u128);

cfg_if! {
    if #[cfg(target_endian = "little")] {
        macro_rules! impl_AsLsfSlice_uNN {
            ($pu_native:ident, $pu_out:ident) => {
                paste! {
                    impl<const N: usize> [<AsLsfSlice_ $pu_out>] for LeastSignificantFirstPrimitiveUnsignedArray<$pu_native, N>
                    {
                        fn [<as_lsf_slice_ $pu_out>](&self) -> &[$pu_out] {
                            let native_arr: &[$pu_native; N] = self.as_ref();
                            //let native_slice: &[$pu_native] = native_arr.as_slice();
                            let len = size_of::<[$pu_native; N]>()/size_of::<$pu_out>();
                            //let len = native_slice.len()*size_of::<$pu_native>()/size_of::<$pu_out>();
                            let ptr = native_arr as *const _ as *const $pu_out;
                            unsafe { std::slice::from_raw_parts(ptr, len) }
                        }

                        fn [<as_lsf_mut_slice_ $pu_out>](&mut self) -> &mut [$pu_out] {
                            let native_arr: &mut [$pu_native; N] = self.as_mut();
                            //let native_slice: &mut [$pu_native] = self.as_lsf_mut_slice();
                            //let len = native_slice.len()*size_of::<$pu_native>()/size_of::<$pu_out>();
                            let len = size_of::<[$pu_native; N]>()/size_of::<$pu_out>();
                            let ptr = native_arr as *mut _ as *mut $pu_out;
                            unsafe { std::slice::from_raw_parts_mut(ptr, len) }
                        }
                    }
                }
            };
        }

        //                   native,    out types
        impl_AsLsfSlice_uNN!(   u16,     u8);
        impl_AsLsfSlice_uNN!(   u32,     u8);
        impl_AsLsfSlice_uNN!(   u32,    u16);
        impl_AsLsfSlice_uNN!(   u64,     u8);
        impl_AsLsfSlice_uNN!(   u64,    u16);
        impl_AsLsfSlice_uNN!(   u64,    u32);
        impl_AsLsfSlice_uNN!(  u128,     u8);
        impl_AsLsfSlice_uNN!(  u128,    u16);
        impl_AsLsfSlice_uNN!(  u128,    u32);
        impl_AsLsfSlice_uNN!(  u128,    u64);
    }
}

//=================================================================================================|

#[cfg(any())]
// someday_when_const_generics_are_stable //? TODO error: generic parameters may not be used in const operations
#[cfg(target_endian = "little")]
impl<Elem, const N: usize> std::convert::AsRef<[u8; { N * Elem::SIZE }]>
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
    #[inline]
    fn as_ref(&self) -> &[u8; { N * Elem::SIZE }] {
        unsafe { std::mem::transmute(&self.0) }
    }
}

/* impl<Elem, const N: usize> AsLsfSlicePrimitiveUnsigned
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
{
    type Elem = Elem;

    /// Returns a slice covering the entire least-significant-first primitive unsigned array.
    fn as_lsf_slice(&self) -> &[Self::Elem] {
        self.0.as_slice()
    }

    /// Returns a mutable slice covering the entire least-significant-first primitive unsigned array.
    fn as_lsf_mut_slice(&mut self) -> &mut [Self::Elem] {
        self.0.as_mut_slice()
    }
} */

/*
cfg_if! {
    if #[cfg(target_endian = "little")] {
        impl<Elem, const N: usize> AsLsfSlice_u8 for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
        where
            Elem: PrimitiveUnsigned,
        {
            fn as_lsf_slice_u8(&self) -> &[u8] {
                let len = size_of_val(&self.0);
                let ptr = &self.0 as *const _ as *const u8;
                unsafe { std::slice::from_raw_parts(ptr, len) }
            }

            fn as_lsf_mut_slice_u8(&mut self) -> &mut [u8] {
                let len = size_of_val(&self.0);
                let ptr = &mut self.0 as *mut _ as *mut u8;
                unsafe { std::slice::from_raw_parts_mut(ptr, len) }
            }
        }

        impl<Elem, const N: usize> AsLsfSlice_u16 for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
        where
            Elem: PrimitiveUnsignedAtLeast16,
        {
            fn as_lsf_slice_u16(&self) -> &[u16] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u16>(), 0);
                assert!(std::mem::align_of::<u16>() <= std::mem::align_of_val(&self.0));
                let ptr = &self.0 as *const _ as *const u16;
                unsafe { std::slice::from_raw_parts(ptr, len) }
            }

            /// Returns a mutable slice of least-significant-first `u16`.
            fn as_lsf_mut_slice_u16(&mut self) -> &mut [u16] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u16>(), 0);
                assert!(std::mem::align_of::<u16>() <= std::mem::align_of_val(&self.0));
                let ptr = &mut self.0 as *mut _ as *mut u16;
                unsafe { std::slice::from_raw_parts_mut(ptr, len) }
            }
        }

        impl<Elem, const N: usize> AsLsfSlice_u32 for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
        where
            Elem: PrimitiveUnsignedAtLeast32,
        {
            fn as_lsf_slice_u32(&self) -> &[u32] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u32>(), 0);
                assert!(std::mem::align_of::<u32>() <= std::mem::align_of_val(&self.0));
                let ptr = &self.0 as *const _ as *const u32;
                unsafe { std::slice::from_raw_parts(ptr, len) }
            }

            /// Returns a mutable slice of least-significant-first `u32`.
            fn as_lsf_mut_slice_u32(&mut self) -> &mut [u32] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u32>(), 0);
                assert!(std::mem::align_of::<u32>() <= std::mem::align_of_val(&self.0));
                let ptr = &mut self.0 as *mut _ as *mut u32;
                unsafe { std::slice::from_raw_parts_mut(ptr, len) }
            }
        }

        impl<Elem, const N: usize> AsLsfSlice_u64 for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
        where
            Elem: PrimitiveUnsignedAtLeast64,
        {
            fn as_lsf_slice_u64(&self) -> &[u64] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u64>(), 0);
                assert!(std::mem::align_of::<u64>() <= std::mem::align_of_val(&self.0));
                let ptr = &self.0 as *const _ as *const u64;
                unsafe { std::slice::from_raw_parts(ptr, len) }
            }

            /// Returns a mutable slice of least-significant-first `u64`.
            fn as_lsf_mut_slice_u64(&mut self) -> &mut [u64] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u64>(), 0);
                assert!(std::mem::align_of::<u64>() <= std::mem::align_of_val(&self.0));
                let ptr = &mut self.0 as *mut _ as *mut u64;
                unsafe { std::slice::from_raw_parts_mut(ptr, len) }
            }
        }

        impl<Elem, const N: usize> AsLsfSlice_u128 for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
        where
            Elem: PrimitiveUnsignedAtLeast128,
        {
            fn as_lsf_slice_u128(&self) -> &[u128] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u128>(), 0);
                assert!(std::mem::align_of::<u128>() <= std::mem::align_of_val(&self.0));
                let ptr = &self.0 as *const _ as *const u128;
                unsafe { std::slice::from_raw_parts(ptr, len) }
            }

            /// Returns a mutable slice of least-significant-first `u128`.
            fn as_lsf_mut_slice_u128(&mut self) -> &mut [u128] {
                let len: usize = size_of_val(&self.0);
                assert_eq!(len%std::mem::align_of::<u128>(), 0);
                assert!(std::mem::align_of::<u128>() <= std::mem::align_of_val(&self.0));
                let ptr = &mut self.0 as *mut _ as *mut u128;
                unsafe { std::slice::from_raw_parts_mut(ptr, len) }
            }
        }
    }
}
// */

impl<Elem, const N: usize, T> ::std::cmp::PartialEq<T>
    for LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>
where
    Elem: PrimitiveUnsigned,
    Self: AsLsfSlicePrimitiveUnsigned<Elem = Elem> + AsLsfSlicePrimitiveUnsignedExt,
    T: AsLsfSlicePrimitiveUnsigned<Elem = Elem> + ?Sized,
{
    fn eq(&self, that: &T) -> bool {
        AsLsfSlicePrimitiveUnsignedExt::eq_vartime(self, that)
    }
}
