// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]

//=================================================================================================|

mod features;
pub use features::*;

pub mod limb_type;
pub use limb_type::*;

pub mod module_usedeclset;
pub use module_usedeclset::*;

/// Sub-modules containing `NumericImpl`s.
pub(crate) mod numimpl;

mod type_author;
pub use type_author::*;

/// Registration and description of supported numeric implementations.
mod numimpl_info;
pub use numimpl_info::*;

/// Shared definitions with public API.
pub mod types;
pub use types::*;

pub mod type_info;
pub use type_info::*;

pub mod write;
pub use write::*;

pub mod write_files;
pub use write_files::*;

//=================================================================================================|

use std::ops::Sub;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail, ensure};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};

use crate::build_mod;

//=================================================================================================|

pub fn all_type_authorings() -> Result<&'static Vec<TypeAuthoringInfo>> {
    static V: OnceLock<Vec<TypeAuthoringInfo>> = OnceLock::new();
    let v = V.get_or_init(|| {
        let mut v_out = Vec::new();

        // Just get the log messages out of the way all at once.
        let _ = all_inventoried_numimpl_authors();

        //println!("cargo:warning=DEBUG: vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv all_type_infos() vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv");

        for numimpl_author in all_inventoried_numimpl_authors() {
            //println!("cargo:warning=DEBUG: vvvvvvvvvvvvvvvvvvvvvvvvvvvvv numimpl {}  vvvvvvvvvvvvvvvvvvvvvvvvvvvvv", numimpl_author.crate_name());

            if !numimpl_author.is_enabled() {
                //println!("cargo:warning=DEBUG: numimpl {} is not enabled", numimpl_author.crate_name());
                continue;
            }

            // Making a whole parallel set of types for `zeroize` is doubling the code size.
            // Probably we need to just control this with a feature. //? TODO
            //let mut zeroizeses = vec![false];
            //if numimpl.supports_secure_zeroize {
            //    zeroizeses.push(true);
            //}

            let crate_name: &'static str = numimpl_author.numimpl_struct().crate_name();

            let limb_types_and_module_names = numimpl_author
                .supported_limb_types_and_module_names()
                .collect::<Vec<_>>();

            if limb_types_and_module_names.is_empty() {
                println!("cargo:warning=WARN: No limb types and modules for numeric impl: {:?}", numimpl_author.crate_name());
            }

            for (limb_type, module_name) in limb_types_and_module_names {
                //println!("cargo:warning=DEBUG: vvvvvvvvvvvvvvvvvvvvv {module_name} {limb_type:?} vvvvvvvvvvvvvvvvvvvvv");
                //println!("cargo:warning=DEBUG: module_name: {module_name}");
                //println!("cargo:warning=DEBUG: limb_type: {limb_type}");

                let numimpl_limb = NumimplLimb::new(numimpl_author, limb_type);

                if numimpl_author.supported_bits().is_empty() {
                    println!("cargo:warning=WARN: No \"bits-NNN\" features enabled for numeric impl: {:?}, module: {module_name}, limb type: {limb_type:?}", numimpl_author.crate_name());
                }

                for &subtype in numimpl_author.supported_subtypes().iter() {
                    //println!("cargo:warning=DEBUG: vvvv subtype: {subtype:?} vvvv");

                    'bits: for &bits in numimpl_author.supported_bits().iter() {
                        //println!("cargo:warning=DEBUG: vvvvvvv bits: {bits} vvvvvvv");

                        if let Err(reason) = numimpl_limb.supports_particular_combination_of_subtype_and_bits(subtype, bits) {
                            println!("cargo:warning=Note: For module:{module_name}, skipping particular type because {reason}");
                            continue 'bits;
                        }

                        //println!("cargo:warning=DEBUG: bits: {bits}, using");

                        let zeroize = false; //for &zeroize in zeroizeses.iter() {
                            let module_name = module_name.to_owned().into();

                            let type_info =
                                TypeInfoStruct::new(
                                    &numimpl_author.numimpl,
                                    module_name,
                                    limb_type,
                                    subtype,
                                    bits,
                                    zeroize,
                                );

                            let type_authoring_info = TypeAuthoringInfo {
                                numimpl_author,
                                numimpl_limb,
                                type_info,
                            };

                            //println!("cargo:warning=DEBUG: {}", type_authoring_info.format_line());

                            v_out.push(type_authoring_info);
                        //}

                        //println!("cargo:warning=DEBUG: ^^^^^^^ bits: {bits} ^^^^^^^");
                    }
                    //println!("cargo:warning=DEBUG: ^^^^ subtype: {subtype:?} ^^^^");
                }
                //println!("cargo:warning=DEBUG: ^^^^^^^^^^^^^^^^^^^^^ {module_name} {limb_type:?} ^^^^^^^^^^^^^^^^^^^^^");
            }
            //println!("cargo:warning=DEBUG: ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ numimpl {}  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^", numimpl_author.crate_name());
        }
        //println!("cargo:warning=DEBUG: ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ all_type_infos() ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");

        if v_out.is_empty() {
            println!("cargo:warning=WARN: No types were generated");
        }

        v_out
    });

    Ok(v)
}
