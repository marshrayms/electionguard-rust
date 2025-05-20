// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::manual_assert)]

use std::collections::{BTreeSet, HashMap};

use anyhow::{Context, Result, anyhow, bail, ensure};
use proc_macro2::Ident;

use crate::build_mod::*;
use crate::*;

//=================================================================================================|

#[derive(Default)]
#[repr(transparent)]
pub struct ModuleName_to_UsedeclSet(HashMap<String, UsedeclSet>);

impl ModuleName_to_UsedeclSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_mut<S: std::string::ToString>(&mut self, module_name: S) -> &mut UsedeclSet {
        self.0.entry(module_name.to_string()).or_default()
    }
}

//-------------------------------------------------------------------------------------------------|

#[derive(Default)]
#[repr(transparent)]
pub struct UsedeclSet(pub BTreeSet<Vec<String>>);

impl UsedeclSet {
    pub fn new() -> Self {
        UsedeclSet(BTreeSet::new())
    }

    pub fn insert(&mut self, use_path: &str) {
        let v: Vec<String> = use_path.split("/").map(|s| s.to_owned()).collect();
        self.0.insert(v);
    }
}
