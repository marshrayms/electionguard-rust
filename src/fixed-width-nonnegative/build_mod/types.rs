// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
//? #![allow(non_snake_case)] // This module needs to talk about types in function names
//-

//=================================================================================================|

include!("../shared-inc/types-pub.inc.rs");

include!("../shared-inc/types-nonpub.inc.rs");
pub use __nonpublic::*;

//=================================================================================================|

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};

//=================================================================================================|

impl ToTokens for Subtype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant_ident = format_ident!("{self}");
        tokens.append_all(quote! { Subtype::#variant_ident });
    }
}

//=================================================================================================|
