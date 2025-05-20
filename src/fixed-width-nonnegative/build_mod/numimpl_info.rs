// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::assertions_on_constants)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::manual_assert)]
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail, ensure};
use proc_macro2::{Ident, Literal};
use quote::format_ident;

use crate::build_mod::*;

//=================================================================================================|

impl quote::ToTokens for NumericImplStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let is_enabled = self.is_enabled;
        let all_possibly_supported_limb_primitive_types =
            self.all_possibly_supported_limb_primitive_types;
        let crate_name = self.crate_name;
        let supported_subtypes = self.supported_subtypes;
        let repr_is_fixed_size_limbs_array =
            format_ident!("{}", self.repr_is_fixed_size_limbs_array);
        let can_support_multiple_limb_types =
            format_ident!("{}", self.repr_is_fixed_size_limbs_array);
        let supported_limb_types = self.supported_limb_types;
        let supported_bits = self
            .supported_bits
            .iter()
            .map(|&b| Literal::usize_unsuffixed(b));
        let supports_secure_zeroize = self.supports_secure_zeroize;

        tokens.append_all(quote! {
            __nonpublic::NumericImplStruct {
                is_enabled: #is_enabled,
                all_possibly_supported_limb_primitive_types: &[ #( #all_possibly_supported_limb_primitive_types, )* ],
                crate_name: #crate_name,
                supported_subtypes: &[ #( #supported_subtypes, )* ],
                repr_is_fixed_size_limbs_array: #repr_is_fixed_size_limbs_array,
                can_support_multiple_limb_types: #can_support_multiple_limb_types,
                supported_limb_types: &[ #( #supported_limb_types, )* ],
                supported_bits: &[ #( #supported_bits, )* ],
                supports_secure_zeroize: #supports_secure_zeroize,
            }
        });
    }
}

//=================================================================================================|

#[derive(Debug)]
/// Information about a numeric implementation and how to make an author for a specific type from
/// it.
pub struct NumericImplAuthor {
    pub numimpl: NumericImplStruct,
    pub bx_new_author: fn(&'static TypeInfoStruct) -> Result<Box<dyn Author>>,
}

impl NumericImplAuthor {
    pub fn module_name_base(&self) -> String {
        #[allow(clippy::collapsible_str_replace)] //? TODO?
        self.crate_name().replace('-', "").replace('_', "")
    }

    /// If multiple limb types could possibly be enabled by features, we append the limb type
    /// to the module name. Any new version of any dependent crate can enable a feature and
    /// we don't want the module name format to change.
    pub fn module_name_has_limb_type_appended(&self) -> bool {
        self.numimpl.can_support_multiple_limb_types
    }

    /// All possible module names from this NumericImpl, regardless of whether they
    /// are enabled by configuration.
    pub fn all_possible_module_names(&self) -> Box<[String]> {
        let module_name_base = self.module_name_base();

        if self.module_name_has_limb_type_appended() {
            self.numimpl
                .all_possibly_supported_limb_primitive_types
                .iter()
                .map(move |&limb_type| format!("{module_name_base}_{limb_type}"))
                .collect()
        } else {
            Box::new([module_name_base])
        }
    }

    pub fn supported_limb_types_and_module_names(
        &self,
    ) -> impl Iterator<Item = (LimbType, String)> + use<'_> {
        let module_name_base = self.module_name_base();

        self.supported_limb_types().iter().map(move |&limb_type| {
            let module_name = if self.module_name_has_limb_type_appended() {
                format!("{module_name_base}_{limb_type}")
            } else {
                module_name_base.clone()
            };
            (limb_type, module_name)
        })
    }
}

//-------------------------------------------------------------------------------------------------|

impl NumericImpl for NumericImplAuthor {
    fn numimpl_struct(&self) -> &NumericImplStruct {
        &self.numimpl
    }
}

//-------------------------------------------------------------------------------------------------|
#[repr(transparent)]
pub struct MakeNumericImplInfo(pub fn() -> NumericImplAuthor);

inventory::collect!(MakeNumericImplInfo);

//-------------------------------------------------------------------------------------------------|

pub fn all_inventoried_numimpl_authors() -> impl Iterator<Item = &'static NumericImplAuthor> {
    static V: OnceLock<Vec<NumericImplAuthor>> = OnceLock::new();
    V.get_or_init(|| {
        let mut v_out = Vec::new();

        for make_numimplinfo in inventory::iter::<MakeNumericImplInfo>.into_iter() {
            v_out.push(make_numimplinfo.0());
        }

        if v_out.is_empty() {
            println!("cargo:warning=WARN: No numeric impl features enabled");
        }

        v_out.sort_by(|a, b| a.crate_name().cmp(b.crate_name()));

        v_out
    })
    .iter()
}

pub fn all_possible_module_names_from_all_inventoried_numimpl_authors()
-> impl Iterator<Item = String> {
    all_inventoried_numimpl_authors()
        .flat_map(|numimpl_author| numimpl_author.all_possible_module_names())
}
