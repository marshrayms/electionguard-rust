// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
#![allow(non_snake_case)] // This module needs to talk about types in function names
//-

use crate::build_mod::*;
use crate::*;

//=================================================================================================|

fn all_possibly_supported_limb_primitive_types() -> &'static [LimbType] {
    &[LimbType::u32, LimbType::u64]
}

//-------------------------------------------------------------------------------------------------|

pub(crate) fn bits_supported(bits: usize) -> bool {
    64 <= bits && bits % 64 == 0
}

//-------------------------------------------------------------------------------------------------|

fn supported_bits() -> &'static [usize] {
    static SB: OnceLock<Vec<usize>> = OnceLock::new();

    SB.get_or_init(||{
        let mut v: Vec<usize> = Feature::all_buildrs_knownfeature_bitses_usize().iter().copied()
            .filter(|&bits| bits_supported(bits))
            .collect();
        if v.is_empty() {
            println!("cargo:warning=WARN: Feature \"crypto-bigint\" was enabled without any supported bits (e.g. \"bits-4096\").");
        }
        v.sort();
        v
    })
}

//-------------------------------------------------------------------------------------------------|

fn supported_limb_types() -> &'static [LimbType; 1] {
    //static SLT: [LimbType; 1] = [LimbType::targptr];
    &[LimbType::targptr]
}

//=================================================================================================|

#[rustfmt::skip]
fn make_type_author() -> NumericImplAuthor {

    let is_enabled = false;
    #[cfg(feature = "crypto-bigint")] let is_enabled = true;

    NumericImplAuthor {
        numimpl: NumericImplStruct {
            is_enabled,
            all_possibly_supported_limb_primitive_types: all_possibly_supported_limb_primitive_types(),
            crate_name:                         AuthorCryptoBigint::NUMERIC_IMPL_NAME,
            supported_subtypes:                 &[ Subtype::Nonnegative ], //Subtype::all(),
            repr_is_fixed_size_limbs_array:     true,
            can_support_multiple_limb_types:    false,
            supported_limb_types:               supported_limb_types(),
            supported_bits:                     supported_bits(),
            supports_secure_zeroize:            true,
        },
        bx_new_author,
    }
}

inventory::submit! { MakeNumericImplInfo(make_type_author) }

//-------------------------------------------------------------------------------------------------|

pub fn bx_new_author(type_info: &'static TypeInfoStruct) -> Result<Box<dyn Author>> {
    Ok(Box::new(AuthorCryptoBigint::try_new(type_info)?))
}

//-------------------------------------------------------------------------------------------------|

struct AuthorCryptoBigint {
    type_info: &'static TypeInfoStruct,
}

impl AuthorCryptoBigint {
    const NUMERIC_IMPL_NAME: &str = "crypto-bigint";

    fn try_new(type_info: &'static TypeInfoStruct) -> Result<Self> {
        Ok(AuthorCryptoBigint { type_info })
    }
}

impl TypeInfo for AuthorCryptoBigint {
    fn typeinfo_struct(&self) -> &TypeInfoStruct {
        self.type_info
    }
}

impl TypeInfoFixedSizeLimbArray for AuthorCryptoBigint {}

impl Author for AuthorCryptoBigint {
    fn crate_name(&self) -> CowStaticStr {
        Self::NUMERIC_IMPL_NAME.into()
    }

    fn as_dyn_afsla_opt(&self) -> Option<&dyn AuthorFixedSizeLimbArray> {
        Some(self)
    }

    fn inner_type_module_path_opt(&self) -> Option<TokenStream> {
        Some(quote! { ::crypto_bigint })
    }

    fn inner_type_turbofish(&self) -> TokenStream {
        let u_bits = format!("U{}", self.bits());
        let u_ts = self.inner_type_module_fq(u_bits);

        if self.subtype() == Subtype::Montgomery {
            quote! { ::crypto_bigint::modular::runtime_mod::DynResidue::< u_ts :: LIMBS> }
        } else {
            u_ts
        }
    }

    //---------------------------------------------------------------------------------------------|

    fn accumulate_usedecls(&self, uses_set: &mut UsedeclSet) -> Result<()> {
        uses_set.insert("crypto_bigint/prelude/*");
        Ok(())
    }

    //---------------------------------------------------------------------------------------------|

    fn opt_impl_inherent_optconst_fn_from_le_bytes_arr_body(&self) -> Option<(bool, TokenStream)> {
        Some((
            false,
            quote! {
                Self(Encoding::from_le_bytes(bytes))
            },
        ))
    }

    fn opt_impl_inherent_optconst_fn_from_be_bytes_arr_body(&self) -> Option<(bool, TokenStream)> {
        Some((
            false,
            quote! {
                Self(Encoding::from_be_bytes(bytes))
            },
        ))
    }

    //---------------------------------------------------------------------------------------------|

    fn impl_inherent_optconst_fn_to_le_bytes_arr_body(&self) -> (bool, TokenStream) {
        (
            false,
            quote! {
                Encoding::to_le_bytes(&self.0)
            },
        )
    }

    fn impl_inherent_optconst_fn_to_be_bytes_arr_body(&self) -> (bool, TokenStream) {
        (
            false,
            quote! {
                Encoding::to_be_bytes(&self.0)
            },
        )
    }

    //----------------------------------------------------------- `Author::write_asrefs_and_borrows`

    //---------------------------------------------------------------- `Author::write_stdcloneclone`

    //---------------------------------------------------------- `Author::write_conversions_to_misc`

    //------------------------------------- `Author::write_infallible_convs_from_primitive_unsigned`

    fn inner_type_has_stdconvertfrom_notbigger_primitive_std_u(&self, _src_bits: usize) -> bool {
        true
    }

    //------------------------------------------------- `Author::write_conversions_from_inner`

    fn enable_unrestricted_conversion_from_inner_type(&self) -> bool {
        self.subtype().all_bit_patterns_valid() || self.subtype() == Subtype::Montgomery
    }

    fn inner_type_has_clone(&self) -> bool {
        true
    }

    //-------------------------------------------------------- `Author::write_conversions_from_misc`

    //-------------------------------------------------------- `Author::write_conversions_from_misc`

    //------------------------------------------------------------- `Author::write_conversions_from`

    //-------------------------------------------------------------------------- `Author::write_ops`

    fn opt_impl_inherent_fn_wrapping_add_self_body(&self, rhs: &Ident) -> Option<TokenStream> {
        Some(quote! {
            Self(self.0.wrapping_add(&rhs.0))
        })
    }

    //fn opt_impl_inherent_fn_wrapping_add_u8_body(&self, rhs: &Ident) -> Option<TokenStream> { None }

    //-------------------------------------------------------------- `Author::write_auxiliary_types`

    //------------------------------------------------- `Author::write_inherent_subtype_conversions`

    //fn write_inherent_subtype_conversions(&self, of: &mut dyn std::io::Write) -> Result<()>

    //---------------------------------------------------------------------------------------------|
}

//-------------------------------------------------------------------------------------------------|

impl AuthorFixedSizeLimbArray for AuthorCryptoBigint {
    fn afsla_zero_inner_expr(&self) -> TokenStream {
        let inner_type = self.inner_type_turbofish();
        quote! { #inner_type::ZERO }
    }

    fn afsla_one_inner_expr_opt(&self) -> Option<TokenStream> {
        let inner_type = self.inner_type_turbofish();
        Some(quote! { #inner_type::ONE })
    }

    fn afsla_all_ones_inner_expr(&self) -> TokenStream {
        let inner_type = self.inner_type_turbofish();
        quote! { #inner_type::MAX }
    }
}

//-------------------------------------------------------------------------------------------------|
