// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
#![allow(non_snake_case)] // This module needs to talk about types in function names
//-
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_mut)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

use crate::build_mod::*;
use crate::*;

//=================================================================================================|

pub(crate) const NUMERIC_IMPL_NAME: &str = "basic-array";

//-------------------------------------------------------------------------------------------------|

#[rustfmt::skip]
fn all_possibly_supported_limb_primitive_types() -> &'static [LimbType] {
    &[  LimbType::u8,
        LimbType::u16,
        LimbType::u32,
        LimbType::u64,
        LimbType::u128, ]
}

//-------------------------------------------------------------------------------------------------|

fn figure_supported_limb_types() -> &'static Vec<LimbType> {
    static ASLT: OnceLock<Vec<LimbType>> = OnceLock::new();
    ASLT.get_or_init(|| {
        let mut v: Vec<LimbType> = Feature::all_buildrs_knownfeatures()
            .iter()
            .filter_map(|kf| {
                use KnownFeature::*;
                match kf {
                    basic_array_u8 => Some(LimbType::u8),
                    basic_array_u16 => Some(LimbType::u16),
                    basic_array_u32 => Some(LimbType::u32),
                    basic_array_u64 => Some(LimbType::u64),
                    // basic_array_u128 => Some(LimbType::u128),  TODO temp commented out due to limitations of crate::significant_first::ops::add_returning_carry
                    _ => None,
                }
            })
            .collect();

        if v.is_empty() {
            //println!("cargo:warning=DEBUG: Feature \"basic-array\" was enabled without an explict limb type feature (such as \"basic-array-u32\").");

            let auto_limb = LimbType::pick_one_based_on_target_pointer_width();

            let target_pointer_width = buildrs_target_pointer_width_opt().unwrap();
            //println!("cargo:warning=INFO: Automatically enabling \"basic-array\" limb type '{auto_limb}' based on target pointer width of {target_pointer_width}.");
            v.push(auto_limb);
        } else {
            v.sort();
        }

        v
    })
}

//-------------------------------------------------------------------------------------------------|

#[rustfmt::skip]
fn make_type_author() -> NumericImplAuthor {
    let is_enabled = false;
    #[cfg(feature = "basic-array")] let is_enabled = true;

    NumericImplAuthor {
        numimpl: NumericImplStruct {
            is_enabled,
            all_possibly_supported_limb_primitive_types: all_possibly_supported_limb_primitive_types(),
            crate_name:                         AuthorBasicArray::NUMERIC_IMPL_NAME,
            supported_subtypes:                 Subtype::all(),
            repr_is_fixed_size_limbs_array:     true,
            can_support_multiple_limb_types:    true,
            supported_limb_types:               figure_supported_limb_types(),
            supported_bits:                     Feature::all_buildrs_knownfeature_bitses_usize(),
            supports_secure_zeroize:            true,
        },
        bx_new_author,
    }
}

inventory::submit! { MakeNumericImplInfo(make_type_author) }

//-------------------------------------------------------------------------------------------------|

pub fn bx_new_author(type_info: &'static TypeInfoStruct) -> Result<Box<dyn Author>> {
    Ok(Box::new(AuthorBasicArray::try_new(type_info)?))
}

//-------------------------------------------------------------------------------------------------|

pub struct AuthorBasicArray {
    type_info: &'static TypeInfoStruct,
}

impl AuthorBasicArray {
    const NUMERIC_IMPL_NAME: &str = NUMERIC_IMPL_NAME;

    fn try_new(type_info: &'static TypeInfoStruct) -> Result<Self> {
        Ok(AuthorBasicArray { type_info })
    }
}

impl TypeInfo for AuthorBasicArray {
    fn typeinfo_struct(&self) -> &TypeInfoStruct {
        self.type_info
    }
}

impl TypeInfoFixedSizeLimbArray for AuthorBasicArray {}

impl Author for AuthorBasicArray {
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
        uses_set.insert("crate/power_of_two/*");

        Ok(())
    }

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

    //-------------------------------------------------------- `Author::write_conversions_from_misc`

    //------------------------------------------------------------- `Author::write_conversions_from`

    //-------------------------------------------------------------------------- `Author::write_ops`

    #[allow(clippy::let_and_return)]
    fn opt_impl_inherent_fn_mul_tophalf_body(&self, rhs: &Ident) -> Option<TokenStream> {
        let mut opt_ts = None;
        ////let inner_type = self.inner_type_turbofish();
        //let zero_inner_expr = self.as_dyn_afsla_opt().unwrap().fsla_zero_inner_expr();
        //let add = self.haclrs_module_fq_ident("mul");
        //Some(quote! {
        //    let mut lhs_a = self.0.clone(); //? TODO avoid unnecessary clone //? zeroize?
        //    let mut rhs_a = #rhs.0.clone(); //? TODO avoid unnecessary clone //? zeroize?
        //    let mut res_a = #zero_inner_expr; //? TODO avoid unnecessary zero-init
        //    #add(lhs_a.as_lsf_mut_slice(), rhs_a.as_lsf_mut_slice(), res_a.as_mut_lsf_array());
        //    Self(res_a)
        //})
        opt_ts
    }

    //-------------------------------------------------------------- `Author::write_auxiliary_types`

    fn write_auxiliary_types(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let bits = self.bits();

        if self.subtype() == Subtype::Nonnegative
            && crate::build_mod::numimpl::crypto_bigint::bits_supported(bits)
        {
            writeln!(
                of,
                r##"
/// Precomputed Montgomery parameters of {bits} bits.
pub struct MontgomeryPrecomputation_{bits} {{
    /// Modulus
    m: Nonnegative_{bits},

    /// Next power of two greater than `m`.
    r_nat: TwoRaisedToANonnegativePower,

    /// `r`, mod `m`.
    r: Nonnegative_{bits},

    // Inverse of `r`, mod `m`. I.e., `r*r_inv mod m`. Should be `1`.
    r_inv: Nonnegative_{bits},

    /// `r` expressed in Montgomery form.
    ///
    /// Compat: Crypto-bigint calls this `r2`.
    r_mgf: Montgomery_{bits},
}}

impl MontgomeryPrecomputation_{bits} {{
    pub fn new_opt<T, Elem, const N: usize>(m: &T) -> Option<Self>
    where
        T: ::std::convert::AsRef<LeastSignificantFirstPrimitiveUnsignedArray<Elem, N>>,
        Elem: PrimitiveUnsignedAtLeast8,
    {{
        const BITS: usize = {bits};
        type CryptoBigintUbits = ::crypto_bigint::U{bits};

        let m_lsfpua: &LeastSignificantFirstPrimitiveUnsignedArray<Elem, N> = m.as_ref();
        if !(m_lsfpua.array_size_in_bits() <= BITS) {{
            return None;
        }}

        let Ok(m) = Nonnegative_{bits}::try_from(m_lsfpua) else {{
            return None;
        }};

        let m_le_bytes_arr = m.to_le_bytes_arr(); //? Zeroize?
        let m_cryptobigint_nonnegative = crate::cryptobigint::Nonnegative_{bits}::from_le_bytes_arr(m_le_bytes_arr);

        assert!(BITS <= CryptoBigintUbits::BITS);

        let m_cryptobigint: &CryptoBigintUbits = m_cryptobigint_nonnegative.as_ref();

        let r_nat = TwoRaisedToANonnegativePower::new(BITS);

        let residue_params = ::crypto_bigint::modular::runtime_mod::DynResidueParams::new(m_cryptobigint);
        //debug!("residue_params = {{residue_params:?}}");

        let one_less_than_r = {{
            let mut n = CryptoBigintUbits::MAX;
            n >> CryptoBigintUbits::BITS.saturating_sub(BITS)
        }};

        let mut r_dynresidue = ::crypto_bigint::modular::runtime_mod::DynResidue::new(&one_less_than_r, residue_params);
        r_dynresidue += ::crypto_bigint::modular::runtime_mod::DynResidue::one(residue_params);

        //let r = r_dynresidue.retrieve().clone();
        let r_cryptobigint_nonnegative = crate::cryptobigint::Nonnegative_{bits}::from(r_dynresidue.retrieve());
        let r = Nonnegative_{bits}::from(r_cryptobigint_nonnegative);

        //let r_mgf = r_dynresidue.as_montgomery().clone();
        let r_mgf_cryptobigint_nonnegative = crate::cryptobigint::Nonnegative_{bits}::from(r_dynresidue.as_montgomery());
        let r_mgf_as_plain_nonnegative = Nonnegative_{bits}::from(r_mgf_cryptobigint_nonnegative);
        let r_mgf = Montgomery_{bits}::from_nonnegative_value_that_caller_assures_us_is_in_montgomery_form(r_mgf_as_plain_nonnegative);

        //debug!("R = 2^{bits} (mod p, reg) = {{}}", r);
        //debug!("R = 2^{bits} (mod p, mgf) = {{}}", r_mgf);

        let (r_inv_dynresidue, success) = r_dynresidue.invert();
        assert!(Into::<bool>::into(success), "Couldn't invert 1 << {bits}?");

        let r_inv_cryptobigint_nonnegative = crate::cryptobigint::Nonnegative_{bits}::from(r_inv_dynresidue.retrieve());
        let r_inv = Nonnegative_{bits}::from(r_inv_cryptobigint_nonnegative);
        //debug!("R^-1 (mod p, reg) = {{}}", r_inv);

        //debug!("R^-1 (mod p, mgf) = {{}}", r_inv_res.as_montgomery());

        Some(Self {{ m, r_nat, r, r_inv, r_mgf }})
    }}

    /// Modulus. Must be odd.
    pub fn m(&self) -> &Nonnegative_{bits} {{ &self.m }}

    /// Next power of two greater than `m`.
    pub fn r_nat(&self) -> &TwoRaisedToANonnegativePower {{ &self.r_nat }}

    /// `r`, mod `m`.
    pub fn r(&self) -> &Nonnegative_{bits} {{ &self.r }}

    // Inverse of `r`, mod `m`. I.e., `r*r_inv mod m`. Should be `1`.
    pub fn r_inv(&self) -> &Nonnegative_{bits} {{ &self.r_inv }}

    /// `r` expressed in Montgomery form.
    ///
    /// Compat: Crypto-bigint calls this `r2`.
    pub fn r_mgf(&self) -> &Montgomery_{bits} {{ &self.r_mgf }}
}}

"##
            )?;
        }

        Ok(())
    }

    //------------------------------------------------- `Author::write_inherent_subtype_conversions`

    fn write_inherent_subtype_conversions(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let bits = self.bits();

        use Subtype::*;
        match self.subtype() {
            Nonnegative => {
                writeln!(
                    of,
                    r##"
//
// Inherent subtype conversion methods.
//

// /// To Montgomery form.
// fn into_montgomery(self, mgpc: &MontgomeryPrecomputation_{bits}) -> Montgomery_{bits} {{
//     //? TODO
// }}
"##
                )?;
            }
            Montgomery => {
                writeln!(
                    of,
                    r##"
/// From a `Nonnegative_{bits}` which the caller asserts is actually in Montgomery form.
pub fn from_nonnegative_value_that_caller_assures_us_is_in_montgomery_form(src: Nonnegative_{bits}) -> Self {{
    Self(src.0)
}}
"##
                )?;
            }
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------|

impl AuthorFixedSizeLimbArray for AuthorBasicArray {
    fn afsla_as_dyn_alsfpua_opt(
        &self,
    ) -> Option<&dyn AuthorLeastSignificantFirstPrimitiveUnsignedArray> {
        Some(self)
    }
}

//-------------------------------------------------------------------------------------------------|

impl AuthorLeastSignificantFirstPrimitiveUnsignedArray for AuthorBasicArray {}

//-------------------------------------------------------------------------------------------------|
