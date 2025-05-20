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

fn figure_supported_limb_types() -> &'static Vec<LimbType> {
    static ASLT: OnceLock<Vec<LimbType>> = OnceLock::new();
    let is_enabled = false;
    #[cfg(feature = "basic-array")]
    let is_enabled = true;

    ASLT.get_or_init(|| {
        let mut v: Vec<LimbType> = Feature::all_buildrs_knownfeatures()
            .iter()
            .filter_map(|kf| match kf {
                KnownFeature::hacl_u32 => Some(LimbType::u32),
                KnownFeature::hacl_u64 => Some(LimbType::u64),
                _ => None,
            })
            .collect();

        if v.is_empty() {
            //println!("cargo:warning=DEBUG: Feature \"hacl\" was enabled without an explict limb type feature (such as \"hacl-u32\").");

            let target_pointer_width = buildrs_target_pointer_width_opt().unwrap();
            let auto_limb = if target_pointer_width < 64 {
                LimbType::u32
            } else {
                LimbType::u64
            };

            //println!("cargo:warning=INFO: Automatically enabling \"hacl\" limb type '{auto_limb}' based on target pointer width of {target_pointer_width}.");
            v.push(auto_limb);
        } else {
            v.sort_by_key(|limb_type| limb_type.bits().unwrap());
        }

        v
    })
}

//-------------------------------------------------------------------------------------------------|

#[rustfmt::skip]
fn make_type_author() -> NumericImplAuthor {
    static SB: OnceLock<Vec<usize>> = OnceLock::new();

    let is_enabled = false;
    #[cfg(any(feature = "hacl", feature = "hacl-u32", feature = "hacl-u64"))] let is_enabled = true;

    let supported_bits: &'static [usize] = SB.get_or_init(||{
        let mut v: Vec<usize> = Feature::all_buildrs_knownfeature_bitses_usize().iter().copied()
            .filter(|bits| [
                256,
                4096,
            ].contains(bits))
            .collect();
        if v.is_empty() {
            println!("cargo:warning=WARN: Feature \"hacl\" was enabled without any supported bits (e.g. \"bits-256\").");
        }
        v.sort();
        v
    });

    NumericImplAuthor {
        numimpl: NumericImplStruct {
            is_enabled,
            all_possibly_supported_limb_primitive_types: all_possibly_supported_limb_primitive_types(),
            crate_name:                            AuthorHaclrs::NUMERIC_IMPL_NAME,
            supported_subtypes:                    &[ Subtype::Nonnegative ], //Subtype::all(),
            repr_is_fixed_size_limbs_array:        true,
            can_support_multiple_limb_types: true,
            supported_limb_types:                  figure_supported_limb_types(),
            supported_bits,
            supports_secure_zeroize:               true,
        },
        bx_new_author,
    }
}

inventory::submit! { MakeNumericImplInfo(make_type_author) }

//-------------------------------------------------------------------------------------------------|

pub fn bx_new_author(type_info: &'static TypeInfoStruct) -> Result<Box<dyn Author>> {
    Ok(Box::new(AuthorHaclrs::try_new(type_info)?))
}

//-------------------------------------------------------------------------------------------------|

struct AuthorHaclrs {
    type_info: &'static TypeInfoStruct,
}

impl AuthorHaclrs {
    const NUMERIC_IMPL_NAME: &str = "hacl";

    fn try_new(type_info: &'static TypeInfoStruct) -> Result<Self> {
        Ok(AuthorHaclrs { type_info })
    }

    /* fn limb_array_type(&self) -> TokenStream {
        let limb_type_ident = self.limb_type_ident();
        let cnt_limbs_literal = self.cnt_limbs_literal();
        quote! { [ #limb_type_ident; #cnt_limbs_literal ] }
    } */

    fn haclrs_module_path(&self) -> TokenStream {
        let bits = self.bits();
        let limb_bits = self.limb_bits();
        let bignum_nn = if limb_bits != 64 {
            format_ident!("bignum{bits}_{limb_bits}")
        } else {
            format_ident!("bignum{bits}")
        };
        quote! { :: hacl :: hacl :: #bignum_nn }
    }

    fn haclrs_module_fq_ident<S: ToString>(&self, name: S) -> TokenStream {
        let name_ident = &format_ident!("{}", name.to_string());
        let haclrs_module_path = self.haclrs_module_path();
        quote! { #haclrs_module_path :: #name_ident }
    }
}

impl TypeInfo for AuthorHaclrs {
    fn typeinfo_struct(&self) -> &TypeInfoStruct {
        self.type_info
    }
}

impl TypeInfoFixedSizeLimbArray for AuthorHaclrs {}

impl Author for AuthorHaclrs {
    fn crate_name(&self) -> CowStaticStr {
        Self::NUMERIC_IMPL_NAME.into()
    }

    fn as_dyn_afsla_opt(&self) -> Option<&dyn AuthorFixedSizeLimbArray> {
        Some(self)
    }

    //---------------------------------------------------------------------------------------------|

    fn accumulate_usedecls(&self, uses_set: &mut UsedeclSet) -> Result<()> {
        uses_set.insert("crate/significant_first/AsLsfSlicePrimitiveUnsigned");
        //uses_set.insert("crate/significant_first/AsLsfPrimitiveUnsignedArray");
        uses_set.insert("crate/significant_first/LeastSignificantFirstPrimitiveUnsignedArray");
        Ok(())
    }

    //---------------------------------------------------------------------------------------------|

    //---------------------------------------------------------------------------------------------|

    //----------------------------------------------------------- `Author::write_asrefs_and_borrows`

    //---------------------------------------------------------------- `Author::write_stdcloneclone`

    //---------------------------------------------------------- `Author::write_conversions_to_misc`

    //------------------------------------- `Author::write_infallible_convs_from_primitive_unsigned`

    //------------------------------------------------------- `Author::write_conversions_from_inner`

    fn enable_unrestricted_conversion_from_inner_type(&self) -> bool {
        self.subtype().all_bit_patterns_valid()
    }

    fn inner_type_has_clone(&self) -> bool {
        true
    }

    //------------------------------------------------- `Author::write_conversions_from_misc`

    //------------------------------------------------- `Author::write_conversions_from`

    //------------------------------------------------- `Author::write_ops`

    fn opt_impl_inherent_fn_wrapping_add_self_body(&self, rhs: &Ident) -> Option<TokenStream> {
        let zero_inner_expr = self.alsfpua_zero_inner_expr();
        let add = self.haclrs_module_fq_ident("add");
        Some(quote! {
            let mut lhs_a = self.0.clone(); //? TODO avoid unnecessary clone //? zeroize?
            let mut rhs_a = #rhs.0.clone(); //? TODO avoid unnecessary clone //? zeroize?
            let mut res_a = #zero_inner_expr; //? TODO avoid unnecessary zero-init
            #add(lhs_a.as_lsf_mut_slice(), rhs_a.as_lsf_mut_slice(), res_a.as_mut_lsf_array());
            Self(res_a)
        })
    }
}

//-------------------------------------------------------------------------------------------------|

impl AuthorFixedSizeLimbArray for AuthorHaclrs {
    fn afsla_as_dyn_alsfpua_opt(
        &self,
    ) -> Option<&dyn AuthorLeastSignificantFirstPrimitiveUnsignedArray> {
        Some(self)
    }
}

//-------------------------------------------------------------------------------------------------|

impl AuthorLeastSignificantFirstPrimitiveUnsignedArray for AuthorHaclrs {}

//-------------------------------------------------------------------------------------------------|
