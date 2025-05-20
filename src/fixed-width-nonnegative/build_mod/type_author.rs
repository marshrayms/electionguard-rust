// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(elided_lifetimes_in_paths)]
#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
#![allow(non_snake_case)] // This module needs to talk about types in function names

#[rustfmt::skip] //? TODO: Remove temp development code
use std::{
    borrow::Cow,
    //cell::RefCell,
    //collections::{BTreeSet, BTreeMap},
    //collections::{HashSet, HashMap},
    //hash::{BuildHasher, Hash, Hasher},
    io::{BufRead, Cursor},
    //iter::zip,
    iter::repeat_n,
    //marker::PhantomData,
    path::{Path, PathBuf},
    //process::ExitCode,
    //rc::Rc,
    //str::FromStr,
    sync::{
        Arc,
        //LazyLock,
        //OnceLock,
    },
};

use anyhow::{Context, Result, anyhow, bail, ensure};
//use const_default::ConstDefault;
//use either::Either;
//use futures_lite::future::{self, FutureExt};
//use hashbrown::HashMap;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};
//use rand::{distr::Uniform, Rng, RngCore};
//use serde::{Deserialize, Serialize};
//use static_assertions::{assert_obj_safe, assert_impl_all, assert_cfg, const_assert};
use syn::Lit;
use tracing::{
    debug, error, field::display as trace_display, info, info_span, instrument, trace, trace_span,
    warn,
};

use crate::build_mod::*;
use crate::*;

//=================================================================================================|

pub fn opt_const_ts(is_const: bool) -> TokenStream {
    if is_const {
        quote! { const }
    } else {
        TokenStream::new()
    }
}

//-------------------------------------------------------------------------------------------------|

pub trait Author: TypeInfoExtPm2 {
    /// Check some basic stuff.
    fn verify_basics(&self) -> Result<()> {
        // Verify that if the inner type is a fixed-size limb array, the `AuthorFixedSizeLimbArray`.
        let is_fsla = self.type_info_fsla_opt().is_some();
        let implements_afsla = self.as_dyn_afsla_opt().is_some();
        if is_fsla != implements_afsla {
            if is_fsla {
                bail!(
                    "According to the `TypeInfo`, the type represents a fixed-size limb array, please impl `AuthorFixedSizeLimbArray` and override `Author::as_dyn_afsla_opt()`."
                )
            } else {
                bail!(
                    "According to the `TypeInfo`, the type does not represent a fixed-size limb array. `Author::as_dyn_afsla_opt()` should return `none`."
                )
            }
        }

        Ok(())
    }

    /// The name of the crate providing the numeric implementation.
    fn crate_name(&self) -> CowStaticStr;

    /// Override this if the type is based on a fixed-size limb array.
    /// Simply `impl AuthorFixedSizeLimbArray` and return `Some(self)`.
    fn as_dyn_afsla_opt(&self) -> Option<&dyn AuthorFixedSizeLimbArray> {
        None
    }

    /// Returns the path tokens to refer to the inner type's module,
    /// or `None` if everything's already in scope.
    ///
    /// For example, an implementation might return:
    ///     `Some(quote! { ::bigint_module })`
    ///
    fn inner_type_module_path_opt(&self) -> Option<TokenStream> {
        None
    }

    /// Construct a `TokenStream` that fully-qualifies the provided name from the inner type module.
    fn inner_type_module_fq_ident(&self, name: &Ident) -> TokenStream {
        if let Some(module_path) = self.inner_type_module_path_opt() {
            quote! { #module_path :: #name }
        } else {
            quote! { #name }
        }
    }

    /// A `TokenStream` representing the inner type, possibly an array or fully-qualified name.
    /// For expression context, uses the turbofish operator where necessary.
    fn inner_type_turbofish(&self) -> TokenStream {
        if self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray() {
            let limb_type_ident = self.limb_type_ident();
            let cnt_limbs_literal = Literal::usize_unsuffixed(self.cnt_limbs_opt().unwrap());
            quote! {
                LeastSignificantFirstPrimitiveUnsignedArray::<#limb_type_ident, #cnt_limbs_literal>
            }
        } else {
            panic!("Please override Author::inner_type_turbofish()");
        }
    }

    /// A `TokenStream` representing the inner type, possibly an array or fully-qualified name.
    /// For type contexts where turbofish is not needed.
    fn inner_type(&self) -> TokenStream {
        self.inner_type_turbofish()
    }

    /// Optionally, two `TokenStream`s, when prepended and appended to `self`, produce a &mut to
    /// an inner array type in least-significant-first order.
    fn opt_inner_type_as_lsf_array_mut(&self, self_ident_str: &str) -> Option<TokenStream> {
        if self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray() {
            assert!(
                self.type_info_fsla_opt().is_some() && self.as_dyn_afsla_opt().is_some(),
                "If inner type is a LeastSignificantFirstPrimitiveUnsignedArray, the TypeInfo should represent a FixedSizeLimbArray, please impl AuthorFixedSizeLimbArray and override as_dyn_afsla_opt()."
            );

            let limb_type_ident = self.limb_type_ident();
            let cnt_limbs_literal = Literal::usize_unsuffixed(self.cnt_limbs_opt().unwrap());
            let self_ident = format_ident!("{self_ident_str}");
            Some(
                quote! { std::convert::AsMut::<[#limb_type_ident; #cnt_limbs_literal]>::as_mut(&mut #self_ident.0) },
            )
        } else {
            None
        }
    }

    /// Number of limbs as a literal
    fn cnt_limbs_literal_opt(&self) -> Option<Literal> {
        self.cnt_limbs_opt().map(Literal::usize_unsuffixed)
    }

    //---------------------------------------------------------------------------------------------|

    fn write_whole_file(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let type_info = self.typeinfo_struct();
        let type_ident = self.type_ident();

        of.write_all(crate::GENERATED_FILE_COMMENT_HEADER.as_bytes())
            .map_err(Into::<anyhow::Error>::into)?;

        writeln!(of)?;
        write_comment(of, format!("`{type_ident}`\n\n{type_info:#?}\n\n").as_str())?;

        self.write_auxiliary_types(of)?;

        self.write_struct_and_impl_inherent(of)?;

        self.write_impl_std_fmt_UpperLowerHex(of)?;
        self.write_impl_std_fmt_Debug_Display(of)?;
        self.write_asrefs_and_borrows(of)?;
        self.write_stdcloneclone(of)?;
        self.write_stdconvertfrom_ref_self(of)?;
        self.write_conversions_to_misc(of)?;
        self.write_infallible_convs_from_primitive_unsigned(of)?;
        self.write_conversions_from_inner(of)?;
        self.write_conversions_from_misc(of)?;
        self.write_significantfirst_trait_impls(of)?;

        // write_conversions_from is called separately with a rhs_type_info: &TypeInfoStruct

        //? TODO Serde stuff

        Ok(())
    }

    //---------------------------------------------------------------------------------------------|

    fn accumulate_usedecls_top(&self, uses_set: &mut UsedeclSet) -> Result<()> {
        self.accumulate_usedecls(uses_set)?;

        if self.typeinfo_struct().zeroize() {
            uses_set.insert("zeroize/Zeroize");
        }

        if self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray() {
            uses_set.insert("crate/primitive_unsigned/*");
        }

        Ok(())
    }

    fn accumulate_usedecls(&self, uses_set: &mut UsedeclSet) -> Result<()> {
        Ok(())
    }

    //-------------------------------------------------------------- `Author::write_auxiliary_types`

    fn write_auxiliary_types(&self, of: &mut dyn std::io::Write) -> Result<()> {
        writeln!(
            of,
            "\n//\n// Auxiliary types would go here, if there were any.\n//"
        )?;
        Ok(())
    }

    //---------------------------------------------------------------------------------------------|

    #[rustfmt::skip]
    fn write_struct_and_impl_inherent(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let type_info = self.typeinfo_struct();
        let type_ident = self.type_ident();

        let derive_zeroize = if type_info.zeroize() {
            quote! { #[derive(Zeroize)] }
        } else {
            TokenStream::new()
        };

        let inner_type = self.inner_type();

        write_tokens!(of, quote! {
            #derive_zeroize
            #[repr(transparent)]
            pub struct #type_ident(#inner_type);
        })?;

        writeln!(of, "\nimpl {type_ident} {{")?;

        self.write_impl_inherent_items(of)?;

        writeln!(of, "\n}} // impl {type_ident}").map_err(Into::<anyhow::Error>::into)
    }

    //---------------------------------------------------------------------------------------------|

    #[rustfmt::skip]
    fn write_impl_inherent_items(&self, of: &mut dyn std::io::Write) -> Result<()> {
        self.write_impl_inherent_underlying_numeric_impl(of)?;

        if self.can_const_construct() {
            write_tokens!(of, quote! {
                /// The smallest value that can be represented by this type.
                /// All bits are cleared to `0`.
                pub const MIN: Self = Self::zero();

                /// The largest value that can be represented by this type.
                /// All bits are set to `1`.
                pub const MAX: Self = Self::all_ones();
            })?;
        }

        let opt_const = opt_const_ts(self.can_const_construct());

        let bits = Literal::usize_unsuffixed(self.typeinfo_struct().bits());
        let bytes = Literal::usize_unsuffixed(self.typeinfo_struct().bytes_needed());

        write_tokens!(of, quote! {
            /// The size in bits represented by this type.
            pub const BITS: usize = #bits;
        })?;

        write_tokens!(of, quote! {
            /// The size in bytes represented by this type.
            pub const BYTES: usize = #bytes;
        })?;

        let opt_cnt_limbs = self.typeinfo_struct().cnt_limbs_opt();
        if let Some(cnt_limbs) = opt_cnt_limbs {
            let cnt_limbs = Literal::usize_unsuffixed(cnt_limbs);
            let limb_type_ident = self.limb_type_ident();
            write_tokens!(of, quote! {
                /// The number of limbs used to represent this type.
                pub const CNT_LIMBS: usize = #cnt_limbs;
                /// A limb value of 0.
                pub const LIMB_ZERO: #limb_type_ident = 0;
            })?;
        }

        if let Some(limb_bits) = self.typeinfo_struct().limb_bits_opt() {
            let limb_bits = Literal::usize_unsuffixed(limb_bits);
            write_tokens!(of, quote! {
                /// The size of a limb, in bits.
                pub const LIMB_BITS: usize = #limb_bits;
            })?;
        }

        if let Some(limb_bytes) = self.typeinfo_struct().limb_bytes_exact_opt() {
            let limb_bytes = Literal::usize_unsuffixed(limb_bytes);
            write_tokens!(of, quote! {
                /// The size of a limb, in bytes.
                pub const LIMB_BYTES: usize = #limb_bytes;
            })?;
        }

        if let Some((is_const, body)) = self.impl_inherent_optconst_fn_zero_body_opt() {
            assert!(is_const || !self.can_const_construct());
            let opt_const = opt_const_ts(is_const);
            write_tokens!(of, quote! {
                /// All bits are cleared to `0`.
                /// The smallest value that can be represented by this type.
                pub #opt_const fn zero() -> Self {
                    #body
                }
            })?;
        }

        if let Some((is_const, body)) = self.impl_inherent_optconst_fn_one_body_opt() {
            assert!(is_const || !self.can_const_construct());
            let opt_const = opt_const_ts(is_const);
            write_tokens!(of, quote! {
                /// The least-significant bit is set to `1`, all other bits are cleared to `0`.
                pub #opt_const fn one() -> Self {
                    #body
                }
            })?;
        }

        if let Some((is_const, body)) = self.impl_inherent_optconst_fn_all_ones_body_opt() {
            assert!(is_const || !self.can_const_construct());
            let opt_const = opt_const_ts(is_const);
            write_tokens!(of, quote! {
                /// All bits are set to `1`.
                /// The largest value that can be represented by this type.
                pub #opt_const fn all_ones() -> Self {
                    #body
                }
            })?;
        }

        if let Some((is_const, body)) = self.opt_impl_inherent_optconst_fn_from_le_bytes_arr_body() {
            let opt_const = opt_const_ts(is_const);
            write_tokens!(of, quote! {
                /// Construct from fixed-size byte array in little endian.
                pub #opt_const fn from_le_bytes_arr(bytes: [u8; Self::BYTES]) -> Self {
                    #body
                }
            })?;
        }

        if let Some((is_const, body)) = self.opt_impl_inherent_optconst_fn_from_be_bytes_arr_body() {
            let opt_const = opt_const_ts(is_const);
            write_tokens!(of, quote! {
                /// Construct from fixed-size byte array in big endian.
                pub #opt_const fn from_be_bytes_arr(bytes: [u8; Self::BYTES]) -> Self {
                    #body
                }
            })?;
        }

        // `Self::limbs_lsf_iter()`, `Self::limbs_msf_iter()`
        {
            let mut opt_bodies = self.limbs_lsf_msf_iter_bodies_opt();

            if opt_bodies.is_none() && self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray() {
                opt_bodies = Some((
                    quote! { self.0.as_lsf_slice().iter().cloned() },
                    quote! { self.0.as_lsf_slice().iter().rev().cloned() },
                ));
            }

            if let Some(bodies) = opt_bodies {
                let return_types = self.limbs_lsf_msf_iter_return_types();
                for (lsf_msf, least_most, decl, body) in [
                    ( "lsf", "least", return_types.0, bodies.0 ),
                    ( "msf", "most",  return_types.1, bodies.1 ),
                ] {
                    let comment = format!("Iterator over limb values, in {least_most}-significant-first order.");
                    write_doc_comment(of, comment.as_str())?;
                    let fn_ident = format_ident!("limbs_{lsf_msf}_iter");
                    write_tokens!(of, quote! {
                        pub fn #fn_ident(&self) -> #decl {
                            #body
                        }
                    })?;
                }
            }
        }

        // `Self::to_le_bytes_arr()`
        {
            let (is_const, body) = self.impl_inherent_optconst_fn_to_le_bytes_arr_body();
            let opt_const = opt_const_ts(is_const);
            write_tokens!(of, quote! {
                /// Return the numeric representation as a little endian fixed-size byte array.
                pub #opt_const fn to_le_bytes_arr(&self) -> [u8; Self::BYTES] {
                    #body
                }
            })?;
        }

        // `Self::to_be_bytes_arr()`
        {
            let (is_const, body) = self.impl_inherent_optconst_fn_to_be_bytes_arr_body();
            let opt_const = opt_const_ts(is_const);
            write_tokens!(of, quote! {
                /// Return the numeric representation as a big endian fixed-size byte array.
                pub #opt_const fn to_be_bytes_arr(&self) -> [u8; Self::BYTES] {
                    #body
                }
            })?;
        }

        // `Self::into_inner()`
        if let Some((is_const, body)) = self.impl_inherent_optconst_fn_into_inner_body_opt()
        {
            let opt_const = opt_const_ts(is_const);
            let inner_type = self.inner_type();
            write_tokens!(of, quote! {
                /// Convert `self` into its inner type, `#inner_type`.
                pub #opt_const fn into_inner(self) -> #inner_type {
                    #body
                }
            })?;
        }

        self.write_inherent_ops(of)?;
        self.write_inherent_subtype_conversions(of)?;

        Ok(())
    }

    #[rustfmt::skip]
    fn write_impl_inherent_underlying_numeric_impl(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let underlying_impl_crate_name_str = self.crate_name();
        write_tokens!(of, quote! {
            /// The underlying numeric implementation crate name.
            pub const UNDERLYING_IMPL_CRATE_NAME: &'static str = #underlying_impl_crate_name_str;
        })
    }

    //---------------------------------------------------------------------------------------------|

    fn can_const_construct(&self) -> bool {
        true
    }

    fn impl_inherent_optconst_fn_zero_body_opt(&self) -> Option<(bool, TokenStream)> {
        let expr = if let Some(afsla) = self.as_dyn_afsla_opt() {
            afsla.afsla_zero_inner_expr()
        } else {
            warn!(
                "TODO: Override `Author*::*_zero_inner_expr` for {}::{}",
                self.module_name(),
                self.type_name()
            );
            return None;
        };
        Some((true, quote! { Self(#expr) }))
    }

    fn impl_inherent_optconst_fn_one_body_opt(&self) -> Option<(bool, TokenStream)> {
        let opt_expr = if let Some(afsla) = self.as_dyn_afsla_opt() {
            afsla.afsla_one_inner_expr_opt()
        } else {
            None
        };

        let Some(expr) = opt_expr else {
            warn!(
                "TODO: Override `Author*::*_one_inner_expr_opt` for {}::{}",
                self.module_name(),
                self.type_name()
            );
            return None;
        };

        Some((true, quote! { Self(#expr) }))
    }

    fn impl_inherent_optconst_fn_all_ones_body_opt(&self) -> Option<(bool, TokenStream)> {
        let expr = if let Some(afsla) = self.as_dyn_afsla_opt() {
            afsla.afsla_all_ones_inner_expr()
        } else {
            warn!(
                "TODO: Override `Author*::*_all_ones_inner_expr` for {}::{}",
                self.module_name(),
                self.type_name()
            );
            return None;
        };

        Some((true, quote! { Self(#expr) }))
    }

    //---------------------------------------------------------------------------------------------|

    fn opt_impl_inherent_optconst_fn_from_le_bytes_arr_body(&self) -> Option<(bool, TokenStream)> {
        if self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray() {
            assert!(
                self.type_info_fsla_opt().is_some() && self.as_dyn_afsla_opt().is_some(),
                "If inner type is a LeastSignificantFirstPrimitiveUnsignedArray, the TypeInfo should represent a FixedSizeLimbArray, please impl AuthorFixedSizeLimbArray and override as_dyn_afsla_opt()."
            );

            //? TODO consider making an optional trait (derived from AuthorFixedSizeLimbArray) for inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray
            let inner_type = self.inner_type_turbofish();
            Some((
                false,
                quote! { Self(#inner_type::try_from_le_bytes_arr(bytes).unwrap()) },
            ))
        } else {
            None
        }
    }

    fn opt_impl_inherent_optconst_fn_from_be_bytes_arr_body(&self) -> Option<(bool, TokenStream)> {
        if self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray() {
            assert!(
                self.type_info_fsla_opt().is_some() && self.as_dyn_afsla_opt().is_some(),
                "If inner type is a LeastSignificantFirstPrimitiveUnsignedArray, the TypeInfo should represent a FixedSizeLimbArray, please impl AuthorFixedSizeLimbArray and override as_dyn_afsla_opt()."
            );

            let inner_type = self.inner_type_turbofish();
            Some((
                false,
                quote! { Self(#inner_type::try_from_be_bytes_arr(bytes).unwrap()) },
            ))
        } else {
            None
        }
    }

    //------------------------------------------------------ `Author::limbs_lsf_msf_iter_bodies_opt`

    fn limbs_lsf_msf_iter_bodies_opt(&self) -> Option<(TokenStream, TokenStream)> {
        None
    }

    fn limbs_lsf_msf_iter_return_types(&self) -> (TokenStream, TokenStream) {
        let limb_type_ident = self.limb_type_ident();
        (
            quote! { impl std::iter::Iterator<Item = #limb_type_ident> + '_ },
            quote! { impl std::iter::Iterator<Item = #limb_type_ident> + '_ },
        )
    }

    //------------------------------------------ `Author::impl_inherent_optconst_fn_to_le_bytes_arr`

    fn impl_inherent_optconst_fn_to_le_bytes_arr_body(&self) -> (bool, TokenStream) {
        let ts = quote! {
            //? TODO on a little-endian target when limbs are in lsf order, this could be just a memcpy
            let mut a = [0_u8; Self::BYTES];
            let mut it_dst = a.chunks_exact_mut(Self::LIMB_BYTES);
            for limb in self.limbs_lsf_iter() {
                it_dst.next().unwrap().copy_from_slice(&limb.to_le_bytes());
            }
            debug_assert!(it_dst.next().is_none());
            a
        };
        (false, ts)
    }

    fn impl_inherent_optconst_fn_to_be_bytes_arr_body(&self) -> (bool, TokenStream) {
        let ts = quote! {
            //? TODO on a big-endian target when limbs are in msf order, this could be just a memcpy
            let mut a = [0_u8; Self::BYTES];
            let mut it_dst = a.chunks_exact_mut(Self::LIMB_BYTES);
            for limb in self.limbs_msf_iter() {
                it_dst.next().unwrap().copy_from_slice(&limb.to_be_bytes());
            }
            debug_assert!(it_dst.next().is_none());
            a
        };
        (false, ts)
    }

    //-------------------------------------- `Author::impl_inherent_optconst_fn_into_inner_body_opt`

    fn can_const_into_inner(&self) -> bool {
        /*
        // `Zeroize` means the type impls the `Drop` trait, which is needed for `into_inner()`,
        // which means it has a desctructor, which means `into_inner()` can't be `const`.
        ! self.zeroize()
        // */
        false
    }

    fn impl_inherent_optconst_fn_into_inner_body_opt(&self) -> Option<(bool, TokenStream)> {
        Some((self.can_const_into_inner(), quote! { self.0 }))
    }

    //---------------------------------------------------------------------------------------------|

    #[rustfmt::skip]
    #[allow(non_snake_case)]
    fn write_impl_std_fmt_UpperLowerHex(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let type_ident = self.type_ident();
        let body_upper = self.impl_stdfmt_UpperLowerHex_fmt_body(true);
        let body_lower = self.impl_stdfmt_UpperLowerHex_fmt_body(false);

        write_tokens!(of, "\n", quote! {
            /// To uppercase hex.
            impl std::fmt::UpperHex for #type_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    #body_upper
                }
            }
        })?;

        write_tokens!(of, "\n", quote! {
            /// To lowercase hex.
            impl std::fmt::LowerHex for #type_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    #body_lower
                }
            }
        })
    }

    #[rustfmt::skip]
    #[allow(non_snake_case)]
    fn impl_stdfmt_UpperLowerHex_fmt_body(&self, upper: bool) -> TokenStream {
        let upper_lower = if upper { "upper" } else { "lower" };
        let upper_lower = format_ident!("{upper_lower}");

        quote! {
            use zeroize::Zeroize;

            if f.alternate() { f.write_str("0x")?; }
            let mut aby: [ u8; Self::BYTES ] = self.to_be_bytes_arr();
            let mut ahx = [ 0u8; Self::BYTES*2 ];
            let result = f.write_str(
                ::base16ct:: #upper_lower ::encode_str(&aby, &mut ahx).unwrap()
            );
            aby.zeroize();
            ahx.zeroize();
            result
        }
    }

    fn write_impl_std_fmt_Debug_Display(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let type_name = self.type_name();
        let type_ident = self.type_ident();
        let wid = Literal::usize_unsuffixed(self.bits().div_ceil(4));

        write_tokens!(
            of,
            "\n",
            quote! {
                /// std::fmt::Debug
                impl std::fmt::Debug for #type_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{} {{ {self:0wid$X} }}", #type_name, wid=#wid)
                    }
                }

                /// std::fmt::Display
                impl std::fmt::Display for #type_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        std::fmt::Debug::fmt(self, f)
                    }
                }
            }
        )
    }

    //---------------------------------------------------- `Author::write_asrefs_and_borrows`

    fn write_asrefs_and_borrows(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let inner_type = self.inner_type();
        let type_ident = self.type_ident();

        write_tokens!(
            of,
            "\n",
            quote! {
                impl std::convert::AsRef<#inner_type> for #type_ident {
                    #[inline] fn as_ref(&self) -> &#inner_type { &self.0 } }
            }
        )?;

        /*
        write_tokens!(of, "\n", quote! {
            impl std::borrow::Borrow<BigUint> for #type_ident {
                #[inline]
                fn borrow(&self) -> &BigUint {
                    &self.0
                }
            }
        })?;
        // */

        Ok(())
    }

    //--------------------------------- `Author::write_stdcloneclone`

    fn write_stdcloneclone(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let src_val_ident = format_ident!("src_val");
        let type_ident = self.type_ident();

        let body = self.impl_stdcloneclone_body();

        write_tokens!(
            of,
            "\n",
            quote! {
                impl std::clone::Clone for #type_ident {
                    fn clone(&self) -> Self {
                        #body
                    }
                }
            }
        )
    }

    fn impl_stdcloneclone_body(&self) -> TokenStream {
        quote! {
            Self(self.0.clone())
        }
    }

    //--------------------------------- `Author::write_stdconvertfrom_ref_self`

    fn write_stdconvertfrom_ref_self(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let src_ident = format_ident!("src");
        let type_ident = self.type_ident();
        if let Some(body) = self.impl_stdconvertfrom_ref_self_body_opt(&src_ident) {
            write_doc_comment(
                of,
                "\nInfallible conversion from `&Self`.\nBasically the same thing as `std::clone::Clone`.",
            )?;
            write_tokens!(
                of,
                "",
                quote! {
                impl std::convert::From<&Self> for #type_ident {
                    fn from(#src_ident: &Self) -> Self {
                        #body
                    }
                }}
            )
        } else {
            writeln!(of, "\n//\n// No implementation of `std::convert::From<&Self>` was generated for `{type_ident}`.\n//")
                .map_err(Into::into)
        }
    }

    fn impl_stdconvertfrom_ref_self_body_opt(&self, src_ident: &Ident) -> Option<TokenStream> {
        Some(quote! { Self(#src_ident.0.clone()) })
    }

    //------------------------------------------------- `Author::write_conversions_to_misc`

    fn write_conversions_to_misc(&self, of: &mut dyn std::io::Write) -> Result<()> {
        Ok(())
    }

    //--------------------------------- `Author::write_infallible_convs_from_primitive_unsigned`

    fn write_infallible_convs_from_primitive_unsigned(
        &self,
        of: &mut dyn std::io::Write,
    ) -> Result<()> {
        let src_val_ident = format_ident!("src_val");
        let type_ident = self.type_ident();

        let src_bits_iter = [8, 16, 32, 64, 128_usize]
            .into_iter()
            .filter(|&sb| sb <= self.bits());

        for src_bits in src_bits_iter {
            let src_unnn_type_ident = format_ident!("u{src_bits}");

            if self.inner_type_has_stdconvertfrom_notbigger_primitive_std_u(src_bits) {
                write_tokens!(
                    of,
                    format!(
                        "\n/// Conversion from primitive type [`{src_unnn_type_ident}`] is lossless and always succeeds.\n"
                    ),
                    quote! {
                        impl std::convert::From<#src_unnn_type_ident> for #type_ident {
                            #[inline]
                            fn from(#src_val_ident: #src_unnn_type_ident) -> Self {
                                Self(std::convert::From::<#src_unnn_type_ident>::from(#src_val_ident))
                            }
                        }
                    }
                )?;
            } else if let Some(body) = self
                .opt_impl_stdconvertfrom_notbigger_primitive_std_unnn_body(
                    src_bits,
                    &src_val_ident,
                    &src_unnn_type_ident,
                )
            {
                write_tokens!(
                    of,
                    format!(
                        "\n/// Conversion from primitive type [`{src_unnn_type_ident}`] is lossless and always succeeds.\n"
                    ),
                    quote! {
                        impl std::convert::From<#src_unnn_type_ident> for #type_ident {
                            #[inline]
                            fn from(#src_val_ident: #src_unnn_type_ident) -> Self {
                                #body
                            }
                        }
                    }
                )?;
            } else {
                writeln!(
                    of,
                    "\n//\n// No implementation of `std::convert::From<{src_unnn_type_ident}>` was generated.\n// TODO\n//"
                )?;
            }
        }

        Ok(())
    }

    /// Override this and return `true` iff inner type `impls` `Nonnegative as std::convert::From<uNN>`
    /// for any `std::uNN` with the same or fewer bits.
    /// You might consider returning `type_info().conversion_is_lossless_from(rhs_type_info)`.
    fn inner_type_has_stdconvertfrom_notbigger_primitive_std_u(&self, src_bits: usize) -> bool {
        false
    }

    /// Override this and return `Some` if inner type returns `false` for `opt_inner_type_as_lsf_array_mut()`.
    #[allow(clippy::question_mark)] //?? TODO
    fn opt_impl_stdconvertfrom_notbigger_primitive_std_unnn_body(
        &self,
        src_bits: usize,
        src_ident: &Ident,
        src_unnn_ident: &Ident,
    ) -> Option<TokenStream> {
        assert!(src_bits <= self.bits());

        let Some(inner_type_as_lsf_array_mut) = self.opt_inner_type_as_lsf_array_mut("self_")
        else {
            return None;
        };

        let Some(limb_bits) = self.limb_bits_opt() else {
            return None;
        };
        assert!(limb_bits != 0);
        let limb_bits_lit = Literal::usize_unsuffixed(limb_bits);

        let Some(cnt_limbs) = self.cnt_limbs_opt() else {
            return None;
        };
        assert!(cnt_limbs != 0);
        let cnt_limbs_literal = self.cnt_limbs_literal_opt().unwrap();

        let cnt_limbs_src = src_bits.div_ceil(limb_bits);

        let limb_type_ident = self.limb_type_ident();
        let mut ts = quote! {
            let mut self_ = Self::zero();
            let a: &mut [#limb_type_ident; #cnt_limbs_literal] = #inner_type_as_lsf_array_mut;
        };
        for limb_ix in 0..cnt_limbs_src {
            let limb_ix_lit = Literal::usize_unsuffixed(limb_ix);
            ts.append_all(quote! {
                a[ #limb_ix_lit ] = #src_ident as #limb_type_ident;
            });
            if limb_ix + 1 != cnt_limbs_src {
                ts.append_all(quote! {
                    let #src_ident = #src_ident >> #limb_bits_lit;
                });
            }
        }
        ts.append_all(quote! { self_ });

        Some(ts)
    }

    //------------------------------------------------- `Author::write_conversions_from_inner`

    /// If this returns `true`, we will generate unrestricted conversions from the inner type.
    ///
    /// When overloading, consider returning the result of `self.subtype().all_bit_patterns_valid()`.
    fn enable_unrestricted_conversion_from_inner_type(&self) -> bool {
        false
    }

    /// If we generate unrestricted conversion from inner type, this allows us to clone a reference.
    fn inner_type_has_clone(&self) -> bool {
        false
    }

    /// If enabled, generate unrestricted conversions from the inner type.
    /// Or you can override this method and do something custom.
    fn write_conversions_from_inner(&self, of: &mut dyn std::io::Write) -> Result<()> {
        if self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray() {
            return self.write_conversions_from_LeastSignificantFirstPrimitiveUnsignedArray(of);
        } else if self.enable_unrestricted_conversion_from_inner_type() {
            let src_ident = format_ident!("src");
            let type_ident = self.type_ident();
            let inner_type = self.inner_type();

            write_tokens!(
                of,
                "/// Conversion from `{inner_type}`.\n",
                quote! {
                    impl std::convert::From<#inner_type> for #type_ident {
                        fn from(#src_ident: #inner_type) -> Self {
                            Self(#src_ident)
                        }
                    }
                }
            )?;

            if self.inner_type_has_clone() {
                write_tokens!(
                    of,
                    "\n/// Conversion from `&{inner_type}`.\n",
                    quote! {
                        impl std::convert::From<&#inner_type> for #type_ident {
                            fn from(#src_ident: &#inner_type) -> Self {
                                Self(::std::clone::Clone::clone(#src_ident))
                            }
                        }
                    }
                )?;

                /* This results in:
                conflicting implementations of trait `From<basicarray_u16::Nonnegative_256>` for type `basicarray_u16::Nonnegative_256`
                conflicting implementation in crate `core`:
                - impl<T> From<T> for T;
                write_tokens!(of, "\n/// Conversion from anything that can `AsRef<{inner_type}>`.\n", quote! {
                    impl<T> ::std::convert::From<T> for #type_ident
                    where
                        T: ::std::convert::AsRef<#inner_type>
                    {
                        fn from(#src_ident: &T) -> Self {
                            Self(::std::clone::Clone::clone(#src_ident.as_ref()))
                        }
                    }
                })?;
                // */
            }
        }

        Ok(())
    }

    fn write_conversions_from_LeastSignificantFirstPrimitiveUnsignedArray(
        &self,
        of: &mut dyn std::io::Write,
    ) -> Result<()> {
        assert!(self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray());

        let subtype = self.subtype();
        let numeric_encoding = subtype.numeric_encoding();
        let lhs_type_ident = self.type_ident();
        let src_val_ident = format_ident!("rhs");

        let rhs_type_path: TokenStream =
            quote! { LeastSignificantFirstPrimitiveUnsignedArray<Elem, N> };
        let rhs_type_path_turbofish =
            quote! { LeastSignificantFirstPrimitiveUnsignedArray::<Elem, N> };
        let rhs_type_path_str = rhs_type_path.to_string();

        if !(numeric_encoding == NumericEncoding::NonnegativeBinaryPositional
            && subtype.all_bit_patterns_valid())
        {
            writeln!(
                of,
                "\n//\n// No implementation of `std::convert::From<&{rhs_type_path}>` was generated for `&{lhs_type_ident}`.\n//"
            )?;
            return Ok(());
        }

        let inner_type = self.inner_type_turbofish();

        write_tokens!(
            of,
            "\n",
            quote! {
                impl<Elem, const N: usize> std::convert::TryFrom<&LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>> for #lhs_type_ident
                where
                    Elem: PrimitiveUnsignedAtLeast8,
                {
                    type Error = anyhow::Error;
                    fn try_from(#src_val_ident: &#rhs_type_path) -> Result<Self, Self::Error> {
                        if Self::BITS != #rhs_type_path_turbofish::BITS {
                            anyhow::bail!("{}", concat!("Narrowing and widening conversions from `", #rhs_type_path_str, "` are not implemented yet //? TODO"));
                        }
                        let lsf_iter = crate::significant_first::AsLsfSlicePrimitiveUnsignedExt::as_lsf_iter_u8(#src_val_ident);
                        let lsfpua = #inner_type::try_from_le_iter_u8(lsf_iter)?;
                        Ok(Self(lsfpua))
                    }
                }
            }
        )?;

        write_tokens!(
            of,
            "\n",
            quote! {
                impl<Elem, const N: usize> std::convert::TryFrom<LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>> for #lhs_type_ident
                where
                    Elem: PrimitiveUnsignedAtLeast8,
                {
                    type Error = anyhow::Error;
                    fn try_from(#src_val_ident: LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>) -> Result<Self, Self::Error> {
                        Self::try_from(&#src_val_ident)
                    }
                }
            }
        )?;

        Ok(())
    }

    //------------------------------------------------- `Author::write_conversions_from_misc`

    fn write_conversions_from_misc(&self, of: &mut dyn std::io::Write) -> Result<()> {
        Ok(())
    }

    //------------------------------------------------- `Author::write_conversions_from`

    fn write_conversions_from(
        &self,
        of: &mut dyn std::io::Write,
        rhs_type_info: &TypeInfoStruct,
    ) -> Result<()> {
        let lhs_type_info = self.typeinfo_struct();
        let lhs_type_ident = &lhs_type_info.type_ident();

        let rhs_type_path = if lhs_type_info.module_name() == rhs_type_info.module_name() {
            let rhs_type_ident = &rhs_type_info.type_ident();
            quote! { #rhs_type_ident }
        } else {
            rhs_type_info.full_crate_relative_path()
        };

        let src_val_ident = format_ident!("rhs");

        // We don't `From` convert between different Subtypes unless it's lossless.
        let conversion_is_always_lossless_and_same_encoding =
            lhs_type_info.conversion_is_always_lossless_and_same_encoding_from(rhs_type_info);

        //writeln!(
        //    of,
        //    "\n//\n// Conversion from `{rhs_type_path}` to `{lhs_type_ident}` is{} always (lossless and the same encoding).\n//",
        //    if conversion_is_always_lossless_and_same_encoding { "" } else { " NOT" }
        //)?;

        if conversion_is_always_lossless_and_same_encoding {
            let mut have_stdconvert_from_ref = false;
            if let Some(body) = self.opt_impl_stdconvert_from_ref(rhs_type_info, &rhs_type_path) {
                write_tokens!(
                    of,
                    "\n",
                    quote! {
                        impl std::convert::From<&#rhs_type_path> for #lhs_type_ident {
                            fn from(#src_val_ident: &#rhs_type_path) -> Self {
                                #body
                            }
                        }
                    }
                )?;
                have_stdconvert_from_ref = true;
            } else {
                writeln!(
                    of,
                    "\n//\n// No implementation of `std::convert::From<&{rhs_type_path}>` was generated for `{lhs_type_ident}`.\n//"
                )?;
            }

            if let Some(body) = self.opt_impl_stdconvert_from(
                rhs_type_info,
                &rhs_type_path,
                have_stdconvert_from_ref,
            ) {
                write_tokens!(
                    of,
                    "\n",
                    quote! {
                        impl std::convert::From<#rhs_type_path> for #lhs_type_ident {
                            fn from(#src_val_ident: #rhs_type_path) -> Self {
                                #body
                            }
                        }
                    }
                )?;
            } else {
                writeln!(
                    of,
                    "\n//\n// No implementation of `std::convert::From<{rhs_type_path}>` was generated for `{lhs_type_ident}`.\n//"
                )?;
            }
        } else {
            let mut have_stdconvert_tryfrom_ref = false;
            if let Some(body) = self.opt_impl_stdconvert_tryfrom_ref(rhs_type_info, &rhs_type_path)
            {
                write_tokens!(
                    of,
                    "\n",
                    quote! {
                        impl std::convert::TryFrom<&#rhs_type_path> for #lhs_type_ident {
                            type Error = anyhow::Error;
                            fn try_from(#src_val_ident: &#rhs_type_path) -> Result<Self, Self::Error> {
                                #body
                            }
                        }
                    }
                )?;
                have_stdconvert_tryfrom_ref = true;
            } else {
                writeln!(
                    of,
                    "\n//\n// No implementation of `std::convert::TryFrom<&{rhs_type_path}>` was generated for `&{lhs_type_ident}`.\n//"
                )?;
            }

            if let Some(body) = self.opt_impl_stdconvert_tryfrom(
                rhs_type_info,
                &rhs_type_path,
                have_stdconvert_tryfrom_ref,
            ) {
                write_tokens!(
                    of,
                    "\n",
                    quote! {
                        impl std::convert::TryFrom<#rhs_type_path> for #lhs_type_ident {
                            type Error = anyhow::Error;
                            fn try_from(#src_val_ident: #rhs_type_path) -> Result<Self, Self::Error> {
                                #body
                            }
                        }
                    }
                )?;
            } else {
                writeln!(
                    of,
                    "\n//\n// No implementation of `std::convert::TryFrom<{rhs_type_path}>` was generated for `{lhs_type_ident}`.\n//"
                )?;
            }
        }

        Ok(())
    }

    fn opt_impl_stdconvert_from_ref(
        &self,
        rhs_type_info: &TypeInfoStruct,
        rhs_type_path: &TokenStream,
    ) -> Option<TokenStream> {
        let lhs_bytes = self
            .typeinfo_struct()
            .bytes_exact_opt()
            .unwrap_or(usize::MAX);
        let rhs_bytes = rhs_type_info.bytes_exact_opt().unwrap_or(usize::MAX - 1);
        if lhs_bytes == rhs_bytes {
            let bytes = Literal::usize_unsuffixed(lhs_bytes);
            Some(quote! {
                //? TODO This is a simplistic conversion using bytes. Perhaps it could be improved?
                //let a: [u8; #bytes] = rhs.to_le_bytes_arr();
                //Self::from_le_bytes_arr(a)
                Self::from_le_bytes_arr(rhs.to_le_bytes_arr())
            })
        } else {
            None
        }
    }

    fn opt_impl_stdconvert_from(
        &self,
        rhs_type_info: &TypeInfoStruct,
        rhs_type_path: &TokenStream,
        have_stdconvert_from_ref: bool,
    ) -> Option<TokenStream> {
        if have_stdconvert_from_ref {
            Some(quote! {
                std::convert::From::<&#rhs_type_path>::from(&rhs)
            })
        } else {
            None
        }
    }

    fn opt_impl_stdconvert_tryfrom_ref(
        &self,
        rhs_type_info: &TypeInfoStruct,
        rhs_type_path: &TokenStream,
    ) -> Option<TokenStream> {
        None
    }

    fn opt_impl_stdconvert_tryfrom(
        &self,
        rhs_type_info: &TypeInfoStruct,
        rhs_type_path: &TokenStream,
        have_stdconvert_tryfrom_ref: bool,
    ) -> Option<TokenStream> {
        if have_stdconvert_tryfrom_ref {
            Some(quote! {
                std::convert::TryFrom::<&#rhs_type_path>::try_from(&rhs)
            })
        } else {
            None
        }
    }

    //------------------------------------------------- `Author::write_ops`

    // Override this to provide a body for `pub fn wrapping_add(&self, &Self)`.
    fn opt_impl_inherent_fn_wrapping_add_self_body(&self, rhs: &Ident) -> Option<TokenStream> {
        None
    }

    // Override this to provide a body for `pub fn wrapping_add_u8(&self, u8)`.
    fn opt_impl_inherent_fn_wrapping_add_u8_body(&self, rhs: &Ident) -> Option<TokenStream> {
        None
    }

    // Override this to provide a body for `pub fn mul_tophalf(&self, &Self)`.
    fn opt_impl_inherent_fn_mul_tophalf_body(&self, rhs: &Ident) -> Option<TokenStream> {
        //unimplemented!()
        None
    }

    fn write_inherent_ops(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let rhs_ident = format_ident!("rhs");

        // `pub fn wrapping_add(&self, &Self)`
        {
            let mut opt_body = self.opt_impl_inherent_fn_wrapping_add_self_body(&rhs_ident);

            if opt_body.is_none()
                && self.inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray()
            {
                opt_body = Some(quote! {
                    let lhs = &self.0;
                    let rhs = &#rhs_ident.0;
                    let (lsfpua, _c) = crate::significant_first::ops::add_returning_carry(lhs, rhs);
                    Self(lsfpua)
                });
            }

            //if opt_body.is_none() {
            //    if let Some(afsla) = self.as_dyn_afsla_opt() {
            //        opt_body = Some(quote! { todo!() });
            //    }
            //}

            if let Some(body) = opt_body {
                write_tokens!(
                    of,
                    quote! {
                        /// Wrapping addition of `self` and another `Self`.
                        pub fn wrapping_add(&self, #rhs_ident: &Self) -> Self {
                            #body
                        }
                    }
                )?;
            }
        }

        if let Some(body) = self.opt_impl_inherent_fn_wrapping_add_u8_body(&rhs_ident) {
            write_tokens!(
                of,
                quote! {
                    /// Wrapping addition of `self` and a `u8`.
                    pub fn wrapping_add_u8(&self, #rhs_ident: u8) -> Self {
                        #body
                    }
                }
            )?;
        }
        let rhs_ident = format_ident!("rhs");

        // `pub fn mul_tophalf(&self, &Self)`
        {
            let opt_body = self.opt_impl_inherent_fn_mul_tophalf_body(&rhs_ident);

            if let Some(body) = opt_body {
                write_tokens!(
                    of,
                    quote! {
                        /// Multiplication of `self` and another `Self`, returning only the upper half.
                        pub fn mul_tophalf(&self, #rhs_ident: &Self) -> Self {
                            #body
                        }
                    }
                )?;
            }
        }

        Ok(())
    }

    //------------------------------------------------- `Author::write_inherent_subtype_conversions`

    fn write_inherent_subtype_conversions(&self, of: &mut dyn std::io::Write) -> Result<()> {
        writeln!(
            of,
            "\n//\n// Inherent subtype conversion methods would go here, if there were any.\n//"
        )?;
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------|

/// This is a separate `Ext` trait, so Author can be object-safe.
///
/// These are convenience methods, you wouldn't override these.
pub trait AuthorExt: Author {
    fn inner_type_module_fq<S: ToString>(&self, name: S) -> TokenStream {
        self.inner_type_module_fq_ident(&format_ident!("{}", name.to_string()))
    }

    // /// Gets the `AuthorLeastSignificantFirstPrimitiveUnsignedArray` interface, if supported.
    // /// Don't override this.
    //fn as_dyn_alsfpua_opt(&self) -> Option<&dyn AuthorLeastSignificantFirstPrimitiveUnsignedArray> {
    //    self.as_dyn_afsla_opt().and_then(AuthorFixedSizeLimbArray::afsla_as_dyn_alsfpua_opt)
    //}

    /// This is mainly for compatibility.
    /// You may prefer to just use `if let Some(alsfpua) = self.as_dyn_alsfpua_opt() { ... }` instead.
    /// Don't override this.
    #[allow(non_snake_case)]
    fn inner_type_is_LeastSignificantFirstPrimitiveUnsignedArray(&self) -> bool {
        //self.as_dyn_alsfpua_opt()
        self.as_dyn_afsla_opt()
            .and_then(AuthorFixedSizeLimbArray::afsla_as_dyn_alsfpua_opt)
            .is_some()
    }

    //--------------------------------------------------------- `write_significantfirst_trait_impls`

    /// Impl any traits from the `significant_first` module.
    /// Don't override this.
    fn write_significantfirst_trait_impls(&self, of: &mut dyn std::io::Write) -> Result<()> {
        //if let Some(alsfpua) = self.as_dyn_alsfpua_opt() {
        //    alsfpua.write_significantfirst_trait_impls(of)
        //} else
        if let Some(afsla) = self.as_dyn_afsla_opt() {
            afsla.afsla_write_significantfirst_trait_impls(of)
        } else {
            Ok(())
        }
    }
}

impl<T> AuthorExt for T where T: Author + ?Sized {}

//-------------------------------------------------------------------------------------------------|

pub trait AuthorFixedSizeLimbArray: Author + TypeInfoFixedSizeLimbArray {
    /// Reference to the `TypeInfoFixedSizeLimbArray` interface.
    fn type_info_fsla(&self) -> &dyn TypeInfoFixedSizeLimbArray {
        self.type_info_fsla_opt().unwrap()
    }

    /// Number of limbs as a literal
    fn cnt_limbs_literal(&self) -> Literal {
        Literal::usize_unsuffixed(self.type_info_fsla().cnt_limbs())
    }

    /// Number of limbs as a literal
    fn limb_bits_literal(&self) -> Literal {
        Literal::usize_unsuffixed(self.type_info_fsla().limb_bits())
    }

    /// Override this if the type is based on a `LeastSignificantFirstPrimitiveUnsignedArray`.
    /// Simply `impl AuthorLeastSignificantFirstPrimitiveUnsignedArray` and return `Some(self)`.
    ///
    /// Callers may prefer the method `AuthorExt::as_dyn_alsfpua_opt`.
    fn afsla_as_dyn_alsfpua_opt(
        &self,
    ) -> Option<&dyn AuthorLeastSignificantFirstPrimitiveUnsignedArray> {
        None
    }

    /// Returns an initializer expression for the limb-array-based inner type representing
    /// the value `0`.
    fn afsla_zero_inner_expr(&self) -> TokenStream {
        if let Some(alsfpua) = self.afsla_as_dyn_alsfpua_opt() {
            alsfpua.alsfpua_zero_inner_expr()
        } else {
            //? let cnt_limbs_literal = Literal::usize_unsuffixed(self.cnt_limbs_opt().unwrap());
            let cnt_limbs_literal = self.cnt_limbs_literal();
            quote! { [ 0; #cnt_limbs_literal ] }
        }
    }

    /// Returns an initializer expression for the limb-array-based inner type representing
    /// the value `1`, if it is easily expressible. But returns `None` if it would require
    /// a sequence of statements.
    fn afsla_one_inner_expr_opt(&self) -> Option<TokenStream> {
        if let Some(alsfpua) = self.afsla_as_dyn_alsfpua_opt() {
            alsfpua.alsfpua_one_inner_expr_opt()
        } else {
            let cnt_limbs = self.cnt_limbs();
            match cnt_limbs {
                0 => Some(quote! { [ ] }),
                1..=100 => {
                    use std::iter::{once, repeat};
                    let it = repeat_n(Literal::u8_unsuffixed(0), cnt_limbs - 1)
                        .chain(once(Literal::u8_unsuffixed(1)));
                    Some(quote! { [ #(#it),* ] })
                }
                _ => None,
            }
        }
    }

    /// Returns an initializer expression for the limb-array-based inner type representing
    /// the all-`1`s value.
    fn afsla_all_ones_inner_expr(&self) -> TokenStream {
        if let Some(alsfpua) = self.afsla_as_dyn_alsfpua_opt() {
            alsfpua.alsfpua_all_ones_inner_expr()
        } else {
            let limb_type_ident = self.limb_type_ident();
            //? let cnt_limbs_literal = Literal::usize_unsuffixed(self.cnt_limbs_opt().unwrap());
            let cnt_limbs_literal = self.cnt_limbs_literal();
            quote! { [ #limb_type_ident::MAX; #cnt_limbs_literal ] }
        }
    }
}

//-------------------------------------------------------------------------------------------------|

pub trait AuthorFixedSizeLimbArrayExt: AuthorFixedSizeLimbArray {
    //--------------------------------------------------- `afsla_write_significantfirst_trait_impls`

    /// Impl any traits from the `significant_first` module.
    /// Don't override this.
    fn afsla_write_significantfirst_trait_impls(&self, of: &mut dyn std::io::Write) -> Result<()> {
        if let Some(alsfpua) = self.afsla_as_dyn_alsfpua_opt() {
            alsfpua.alsfpua_write_significantfirst_trait_impls(of)
        } else {
            Ok(())
        }
    }
}

impl<T> AuthorFixedSizeLimbArrayExt for T where T: AuthorFixedSizeLimbArray + ?Sized {}

//-------------------------------------------------------------------------------------------------|

pub trait AuthorLeastSignificantFirstPrimitiveUnsignedArray: AuthorFixedSizeLimbArray {
    fn alsfpua_zero_inner_expr(&self) -> TokenStream {
        let inner_type = self.inner_type_turbofish();
        quote! { #inner_type::ZERO }
    }

    fn alsfpua_one_inner_expr_opt(&self) -> Option<TokenStream> {
        let inner_type = self.inner_type_turbofish();
        Some(quote! { #inner_type::ONE })
    }

    fn alsfpua_all_ones_inner_expr(&self) -> TokenStream {
        let inner_type = self.inner_type_turbofish();
        quote! { #inner_type::MAX }
    }
}

//-------------------------------------------------------------------------------------------------|

pub trait AuthorLeastSignificantFirstPrimitiveUnsignedArrayExt:
    AuthorLeastSignificantFirstPrimitiveUnsignedArray
{
    //------------------------------------------------- `alsfpua_write_significantfirst_trait_impls`

    /// Impl any traits from the `significant_first` module.
    fn alsfpua_write_significantfirst_trait_impls(
        &self,
        of: &mut dyn std::io::Write,
    ) -> Result<()> {
        self.alsfpua_write_impl_stdcmpeq_asref_lsfpua_same_elem(of)?;
        Ok(())
    }

    /// Impl PartialEq and Eq.
    fn alsfpua_write_impl_stdcmpeq_asref_lsfpua_same_elem(
        &self,
        of: &mut dyn std::io::Write,
    ) -> Result<()> {
        let type_ident = self.type_ident();
        let limb_type_ident = self.limb_type_ident();
        let rhs_ident = format_ident!("rhs");
        let lsfpua_param_ident = format_ident!("Lsfpua");
        let lsfpua_path_ts = quote! { crate::significant_first::LeastSignificantFirstPrimitiveUnsignedArray<#limb_type_ident, N> };

        let Some(body) = self.alsfpua_impl_stdcmppartialeq_same_elem_body_opt(
            &rhs_ident,
            &lsfpua_param_ident,
            &lsfpua_path_ts,
            &limb_type_ident,
        ) else {
            writeln!(
                of,
                "\n//\n// No implementation of `std::cmp::PartialEq<Rhs = {lsfpua_path_ts}>` for `{type_ident}`.\n//"
            )?;
            return Ok(());
        };

        write_tokens!(
            of,
            "\n",
            quote! {
                impl ::std::cmp::PartialEq for #type_ident {
                    fn eq(&self, #rhs_ident: &#type_ident) -> bool {
                        #body
                    }
                }
            }
        )?;

        if !self.alsfpua_impl_stdcmppartialeq_asref_is_eq() {
            writeln!(
                of,
                "\n//\n// No implementation of `std::cmp::Eq<Rhs = {lsfpua_path_ts}>` for `{type_ident}`.\n//"
            )?;
            return Ok(());
        }

        write_tokens!(of, "\n", quote! { impl ::std::cmp::Eq for #type_ident { } })?;

        Ok(())
    }

    fn alsfpua_impl_stdcmppartialeq_same_elem_body_opt(
        &self,
        rhs_ident: &Ident,
        lsfpua_param_ident: &Ident,
        lsfpua_path_ts: &TokenStream,
        limb_type_ident: &Ident,
    ) -> Option<TokenStream> {
        Some(quote! {
            use crate::significant_first::AsLsfSlicePrimitiveUnsignedExt;
            self.0.eq_vartime( & #rhs_ident.0)
        })
    }

    /// Override this to return false if `==` is not reflexive, not symmetric, and/or not transitive.
    fn alsfpua_impl_stdcmppartialeq_asref_is_eq(&self) -> bool {
        true
    }
}

impl<T> AuthorLeastSignificantFirstPrimitiveUnsignedArrayExt for T where
    T: AuthorLeastSignificantFirstPrimitiveUnsignedArray + ?Sized
{
}

//-------------------------------------------------------------------------------------------------|
