// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::expect_used)]
#![deny(clippy::manual_assert)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::assertions_on_constants)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)] // Common for these to be generated.
//-
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_mut)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_snake_case)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

//=================================================================================================|

pub mod power_of_two;
pub mod primitive_unsigned;
pub mod significant_first;

/// Info about the the generated types.
pub mod types {
    include!("../shared-inc/types-pub.inc.rs");
    include!("../shared-inc/types-nonpub.inc.rs");
    include!(concat!(env!("OUT_DIR"), "/types_metadata.inc.rs"));
}

// Modules containing the generated types.
include!(concat!(env!("OUT_DIR"), "/generated_types.inc.rs"));

// Generated macros.
include!(concat!(env!("OUT_DIR"), "/generated_macros.inc.rs"));

//-------------------------------------------------------------------------------------------------|

// Tests.
#[cfg(test)]
mod tests;
