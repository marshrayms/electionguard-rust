// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
//? #![allow(non_snake_case)] // This module needs to talk about types in function names
//-

use anyhow::{Context, Result, anyhow, bail, ensure};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};

use crate::build_mod::*;
use crate::*;

//=================================================================================================|

impl LimbType {
    pub fn pick_one_based_on_target_pointer_width() -> Self {
        let target_pointer_width = buildrs_target_pointer_width_opt().unwrap();
        if target_pointer_width < 64 {
            LimbType::u32
        } else {
            LimbType::u64
        }
    }

    pub fn bits(self) -> Result<usize> {
        self.bits_opt()
            .ok_or_else(|| anyhow!("Don't know limb_bits"))
    }

    pub fn try_from_bits(bits: usize) -> Result<Self> {
        use LimbType::*;
        Ok(match bits {
            8 => u8,
            16 => u16,
            32 => u32,
            64 => u64,
            128 => u128,
            _ => bail!("Bits {bits} not supported for `LimbType`"),
        })
    }

    pub fn ident(self) -> Ident {
        self.bits_opt()
            .map(|b| format_ident!("u{b}"))
            .unwrap_or_else(|| format_ident!("___ERROR_limb_bits_UNKNOWN__"))
    }
}

//-------------------------------------------------------------------------------------------------|

impl quote::ToTokens for LimbType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = format_ident!("{self}");
        tokens.append_all(quote! { LimbType::#ident });
    }
}

//-------------------------------------------------------------------------------------------------|
