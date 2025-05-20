// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
//-

use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail, ensure};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};

use crate::build_mod::*;

use crate::types::Subtype;

//=================================================================================================|

impl TypeInfoStruct {
    pub fn new(
        numimpl: &'static NumericImplStruct,
        module_name: CowStaticStr,
        limb_type: LimbType,
        subtype: Subtype,
        bits: usize,
        zeroize: bool,
    ) -> Self {
        assert_eq!(bits % 8, 0, "Expecting bits to be a multiple of 8");

        let module_name_fq = format!("::fixed_width_nonnegative::{module_name}").into();

        let opt_fsla = numimpl.repr_is_fixed_size_limbs_array().then(|| {
            let limb_bits = limb_type.bits_opt().unwrap();

            let cnt_limbs = bits.div_ceil(limb_bits);
            assert_eq!(
                cnt_limbs * limb_bits,
                bits,
                "bits not evenly divisible by limb_bits"
            );

            FixedSizeLimbArr {
                cnt_limbs,
                limb_bits,
            }
        });

        let type_name = {
            let zeroize_suffix = if zeroize { "_Z" } else { "" };
            format!("{subtype}_{bits}{zeroize_suffix}").into()
        };

        let type_name_fq = format!("{module_name_fq}::{type_name}").into();

        Self {
            numimpl,
            module_name,
            module_name_fq,
            type_name,
            type_name_fq,
            limb_type,
            opt_fsla,
            subtype,
            bits,
            zeroize,
        }
    }
}

//-------------------------------------------------------------------------------------------------|

impl quote::ToTokens for FixedSizeLimbArr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cnt_limbs = Literal::usize_unsuffixed(self.cnt_limbs);
        let limb_bits = Literal::usize_unsuffixed(self.limb_bits);

        tokens.append_all(quote! {
            __nonpublic::FixedSizeLimbArr {
                cnt_limbs: #cnt_limbs,
                limb_bits: #limb_bits,
            }
        });
    }
}

//-------------------------------------------------------------------------------------------------|

impl quote::ToTokens for TypeInfoStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let numimpl = proc_macro2::Punct::new('#', proc_macro2::Spacing::Alone);
        let module_name = self.module_name.as_ref();
        let module_name_fq = self.module_name_fq.as_ref();
        let type_name = self.type_name.as_ref();
        let type_name_fq = self.type_name_fq.as_ref();
        let limb_type = self.limb_type;

        let opt_fsla = if let Some(fsla) = self.opt_fsla {
            quote! { Some(#fsla) }
        } else {
            quote! { None }
        };

        let subtype = self.subtype;
        let bits = Literal::usize_unsuffixed(self.bits);
        let zeroize = format_ident!("{}", self.zeroize);

        tokens.append_all(quote! {
            __nonpublic::TypeInfoStruct {
                numimpl: #numimpl,
                module_name: CowStaticStr::Borrowed(#module_name),
                module_name_fq: CowStaticStr::Borrowed(#module_name_fq),
                type_name: CowStaticStr::Borrowed(#type_name),
                type_name_fq: CowStaticStr::Borrowed(#type_name_fq),
                limb_type: #limb_type,
                opt_fsla: #opt_fsla,
                subtype: #subtype,
                bits: #bits,
                zeroize: #zeroize,
            }
        });
    }
}

//=================================================================================================|

/// Reason for negative response from `NumimplLimb::supports_particular_combination_of_subtype_and_bits`.
pub enum SubtypeBitsNotSupportedReason {
    SubtypeNotSupported(Subtype),
    BitsNotSupported(usize),
    NeitherSubtypeNorBitsSupported(Subtype, usize),
    CombinationOfSubtypeAndBits(Subtype, usize),
    BitsNotDivisibleByLimbBits(usize, usize),
}

/// Suitable for use in a message as "because {reason}."
impl std::fmt::Display for SubtypeBitsNotSupportedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SubtypeBitsNotSupportedReason::*;
        match self {
            SubtypeNotSupported(subtype) => write!(f, "Subtype '{subtype}' is not supported"),
            BitsNotSupported(bits) => write!(f, "bits '{bits}' is not supported"),
            NeitherSubtypeNorBitsSupported(subtype, bits) => write!(
                f,
                "neither Subtype '{subtype}' nor bits '{bits}' are supported"
            ),
            CombinationOfSubtypeAndBits(subtype, bits) => write!(
                f,
                "although some configurations having Subtype '{subtype}' are supported, and some configurations having bits '{bits}' are supported, this particular combination is not supported"
            ),
            BitsNotDivisibleByLimbBits(bits, limb_bits) => write!(
                f,
                "bits '{bits}' is not evenly divisible by limb bits '{limb_bits}'"
            ),
        }
    }
}

//-------------------------------------------------------------------------------------------------|

/// A `NumericImpl` configured with a specific `LimbType` it supports.
#[derive(Clone, Copy)]
pub(crate) struct NumimplLimb {
    numimpl_author: &'static NumericImplAuthor,
    limb_type: LimbType,
}

impl NumimplLimb {
    pub(crate) fn new(numimpl_author: &'static NumericImplAuthor, limb_type: LimbType) -> Self {
        Self {
            numimpl_author,
            limb_type,
        }
    }

    /// Returns `Ok(())` if the specified combination of `Subtype` and `bits` is supported,
    /// otherwise returns the reason.
    pub(crate) fn supports_particular_combination_of_subtype_and_bits(
        &self,
        subtype: Subtype,
        bits: usize,
    ) -> Result<(), SubtypeBitsNotSupportedReason> {
        let subtype_supported = self.supported_subtypes().contains(&subtype);
        let bits_supported = self.supported_bits().contains(&bits);
        use SubtypeBitsNotSupportedReason::*;
        match (subtype_supported, bits_supported) {
            (true, false) => Err(SubtypeNotSupported(subtype)),
            (false, true) => Err(BitsNotSupported(bits)),
            (false, false) => Err(NeitherSubtypeNorBitsSupported(subtype, bits)),
            (true, true) => Ok(()),
        }?;

        if let Some(limb_bits) = self.limb_type.bits_opt() {
            let evenly_divisible = limb_bits != 0 && bits % limb_bits == 0;
            if !evenly_divisible {
                Err(BitsNotDivisibleByLimbBits(bits, limb_bits))?;
            }
        }

        Ok(())
    }
}

impl NumericImpl for NumimplLimb {
    fn numimpl_struct(&self) -> &NumericImplStruct {
        self.numimpl_author.numimpl_struct()
    }
}

//=================================================================================================|

/// A "type authoring" is information about an numimpl author and a type it is requested to author.
pub struct TypeAuthoringInfo {
    pub numimpl_author: &'static NumericImplAuthor,
    pub numimpl_limb: NumimplLimb,
    pub type_info: TypeInfoStruct,
}

impl TypeAuthoringInfo {
    pub fn format_line(&self) -> String {
        let numimpl = self.numimpl().crate_name();
        let module = self.module_name();
        let limb_type = self.limb_type();
        let subtype = self.subtype();
        let bits = self.bits();
        let zeroize = self.zeroize();
        format!(
            "numimpl:{numimpl:<13} module:{module:<15} limb_type:{limb_type:<8} subtype:{subtype:<11} bits:{bits:<4} zeroize:{zeroize:<5}"
        )
    }

    pub fn out_filename(&self) -> String {
        format!("{}_{}.rs", self.module_name(), self.type_name())
    }
}

impl TypeInfo for TypeAuthoringInfo {
    fn typeinfo_struct(&self) -> &TypeInfoStruct {
        &self.type_info
    }
}

//=================================================================================================|

/// Additional functionality built on `TypeInfoExt` implementers, this adds some functions that
/// return `proc_macro2` types.
pub trait TypeInfoExtPm2: TypeInfoExt {
    fn module_ident(&self) -> Ident {
        format_ident!("{}", self.typeinfo_struct().module_name())
    }

    fn type_ident(&self) -> Ident {
        format_ident!("{}", self.typeinfo_struct().type_name())
    }

    fn limb_type_ident(&self) -> Ident {
        //? Cache somehow
        self.limb_type().ident()
    }

    fn full_crate_relative_path(&self) -> TokenStream {
        let module_ident = self.module_ident();
        let type_ident = self.type_ident();
        quote! {
            crate::#module_ident::#type_ident
        }
    }
}

impl<T> TypeInfoExtPm2 for T where T: TypeInfoExt + ?Sized {}

//-------------------------------------------------------------------------------------------------|
