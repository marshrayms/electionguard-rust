// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
//? #![allow(non_snake_case)] // This module needs to talk about types in function names
//-
#![allow(clippy::manual_strip)] //? TODO: Remove temp development code
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)]
//? TODO: Remove temp development code
//? #![allow(non_snake_case)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

//use std::borrow::Cow;
//use std::collections::HashSet;
//use std::io::{BufRead, Cursor};
//use std::mem::{size_of, size_of_val};
//use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail, ensure};
use either::Either;

use crate::*;

//=================================================================================================|

#[derive(
    Clone,
    Copy,
    Debug,
    strum::IntoStaticStr,
    strum::Display,
    strum::EnumString
)]
pub enum KnownFeature {
    basic_array,
    basic_array_u8,
    basic_array_u16,
    basic_array_u32,
    basic_array_u64,
    //basic_array_u128, TODO temp comment out due to limitations of crate::significant_first::ops::add_returning_carry
    crypto_bigint,
    default,
    montgomery,
    hacl,
    hacl_u32,
    hacl_u64,
    num_bigint,
    zeroize,
    bits(usize),
}

//=================================================================================================|

#[derive(Debug)]
pub struct Feature(pub Either<KnownFeature, CowStaticStr>);

impl Feature {
    /// All features passed to build.rs invocation via env vars.
    pub fn all_buildrs_features() -> &'static Vec<Feature> {
        static V: OnceLock<Vec<Feature>> = OnceLock::new();

        V.get_or_init(|| {
            let mut v_out = Vec::new();
            for (k, v) in std::env::vars_os() {
                let k = k.to_string_lossy();
                let v = v.to_string_lossy();

                //println!("cargo:warning=DEBUG: Cargo env var '{k}' = '{v}'");

                const CARGO_FEATURE_PREFIX: &str = "CARGO_FEATURE_";
                if k.starts_with(CARGO_FEATURE_PREFIX) {
                    //println!("cargo:warning=DEBUG: Cargo feature env var '{k}' = '{v}'");

                    if v != "1" {
                        println!("cargo:warning=WARN: Cargo feature env var {k} has value '{v}', was expecting '1'.");
                    }

                    let s = k[CARGO_FEATURE_PREFIX.len()..].to_ascii_lowercase();

                    if s.is_empty() {
                        println!("cargo:warning=WARN: Cargo feature env var {k} is missing the feature name.");
                        continue;
                    }

                    const BITS_PREFIX: &str = "bits_";
                    let mut opt_bits_feature = None;

                    if s.starts_with(BITS_PREFIX) {
                        let s_bits = s[BITS_PREFIX.len()..].to_ascii_lowercase();
                        if let Ok(bits) = usize::from_str(&s_bits) {
                            opt_bits_feature = Some(Either::Left(KnownFeature::bits(bits)));
                        } else {
                            println!("cargo:warning=WARN: Cargo feature env var {k} looks suspiciously like a bits_ feature without the number");
                        }
                    }

                    if opt_bits_feature.is_none() {
                        if let Ok(kf) = KnownFeature::from_str(&s) {
                            opt_bits_feature = Some(Either::Left(kf));
                        }
                     }

                    let ekc = opt_bits_feature.unwrap_or_else(|| {
                        Either::Right(s.into())
                    });

                    let f = Feature(ekc);

                    v_out.push(f);
                }
            }
            v_out
        })
    }

    /// All `KnownFeature`s passed to build.rs invocation via env vars.
    pub fn all_buildrs_knownfeatures() -> &'static Vec<KnownFeature> {
        static V: OnceLock<Vec<KnownFeature>> = OnceLock::new();
        V.get_or_init(|| {
            Feature::all_buildrs_features()
                .iter()
                .filter_map(|feature| match feature.0 {
                    either::Left(kf) => {
                        //println!("cargo:warning=DEBUG: KnownFeature: {kf:?}");
                        Some(kf)
                    }
                    either::Right(ref s) => {
                        //println!("cargo:warning=DEBUG: Unknown Feature: {s}");
                        None
                    }
                })
                .collect()
        })
    }

    /// All `KnownFeature::bits(b)` `b: usize` values passed to build.rs invocation via env vars.
    pub fn all_buildrs_knownfeature_bitses_usize() -> &'static Vec<usize> {
        static V: OnceLock<Vec<usize>> = OnceLock::new();
        V.get_or_init(|| {
            let mut v_out = Vec::new();
            for &kf in Feature::all_buildrs_knownfeatures() {
                if let KnownFeature::bits(bits) = kf {
                    v_out.push(bits);
                }
            }

            if v_out.is_empty() {
                println!("cargo:warning=WARN: No \"bits-NNN\" features enabled");
            }

            v_out
        })
    }

    /// All `KnownFeature::bits(b)` features passed to build.rs invocation via env vars.
    pub fn all_buildrs_knownfeature_bitses() -> &'static Vec<KnownFeature> {
        static V: OnceLock<Vec<KnownFeature>> = OnceLock::new();
        V.get_or_init(|| {
            Self::all_buildrs_knownfeature_bitses_usize()
                .iter()
                .map(|&b| KnownFeature::bits(b))
                .collect()
        })
    }
}

//-------------------------------------------------------------------------------------------------|

// Determine target pointer width when built as part of `build.rs`.
#[cfg(not(not_build_rs))]
pub fn buildrs_target_pointer_width_opt() -> Option<usize> {
    use ::std::sync::OnceLock;

    static TPW: OnceLock<Option<usize>> = OnceLock::new();

    fn imp2() -> Result<usize> {
        let tpw_s = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH")?;
        //println!("cargo:warning=DEBUG: CARGO_CFG_TARGET_POINTER_WIDTH: {tpw_s}");

        let mut warned_env_var = false;
        let mut bits = tpw_s
            .parse()
            .map_err(|err| {
                println!("cargo:warning=Env var CARGO_CFG_TARGET_POINTER_WIDTH = {tpw_s}");
                warned_env_var = true;
                println!("cargo:warning=Couldn't parse as usize: {err}");
                err
            })
            .unwrap_or_default();

        if bits < 16 {
            if !warned_env_var {
                println!("cargo:warning=Env var CARGO_CFG_TARGET_POINTER_WIDTH = {tpw_s}");
                warned_env_var = true;
            }

            #[rustfmt::skip]
            cfg_if::cfg_if! {
                if      #[cfg(target_pointer_width =  "16")] { bits =  16 }
                else if #[cfg(target_pointer_width =  "64")] { bits =  64 }
                //else if #[cfg(target_pointer_width = "128")] { bits = 128 }
                else                                         { bits =  32 }
            }

            println!(
                "cargo:warning=Guessing target pointer width from build host cfg(target_pointer_width): {bits}"
            );
        }

        if bits < 16 {
            bail!("Can't figure a plausible target pointer width using any method");
        }

        Ok(bits)
    }

    *TPW.get_or_init(|| {
        let result = imp2();
        if let Err(err) = &result {
            println!("cargo:warning=ERROR: {err}");
        }
        result.ok()
    })
}
