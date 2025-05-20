// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
//? #![allow(non_snake_case)] // This module needs to talk about types in function names
//-
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_mut)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)]
//? TODO: Remove temp development code
//#![allow(non_snake_case)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

//use std::collections::HashSet;
//use std::convert::AsRef;
use std::io::{BufRead, Cursor};
//use std::mem::{size_of, size_of_val};
//use std::path::{Path, PathBuf};
//use std::str::FromStr;
//use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail, ensure};
//use either::Either;
//use log::{debug, error, info, trace, warn};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote};

//use crate::{};
//use crate::*;

//=================================================================================================|

#[macro_export]
macro_rules! write_tokens {
    ($stdiowrite:expr, $token_stream:expr) => {{ $crate::build_mod::write_errfl_tokens((file!(), line!()), $stdiowrite, $token_stream) }};
    ($stdiowrite:expr, $s:expr, $token_stream:expr) => {{
        ::std::io::Write::write_all($stdiowrite, ($s).as_bytes())
            .map_err(Into::<anyhow::Error>::into)
            .and_then(|()| {
                $crate::build_mod::write_errfl_tokens(
                    (file!(), line!()),
                    $stdiowrite,
                    $token_stream,
                )
            })
    }};
}

//-------------------------------------------------------------------------------------------------|

pub fn write_errfl_tokens(
    error_file_line: (&str, u32),
    of: &mut dyn std::io::Write,
    token_stream: TokenStream,
) -> Result<()> {
    let ts = token_stream.to_string();
    let context_f = || {
        format!(
            "invoked at {}:{}:\nvvvv token stream vvvv\n{ts}\n^^^^ token stream ^^^^",
            error_file_line.0, error_file_line.1
        )
    };
    let syntax_tree = syn::parse2(token_stream)
        .context("from syn::parse2")
        .with_context(context_f)?;
    let formatted = prettyplease::unparse(&syntax_tree);
    of.write_all(formatted.as_bytes()).with_context(context_f)
}

//-------------------------------------------------------------------------------------------------|

enum CommentType {
    Basic,
    Doc,
    DocModule,
}

pub fn write_comment(of: &mut dyn std::io::Write, s: &str) -> Result<()> {
    write_comment_impl_(of, CommentType::Basic, s)
}

pub fn write_doc_comment(of: &mut dyn std::io::Write, s: &str) -> Result<()> {
    write_comment_impl_(of, CommentType::Doc, s)
}

pub fn write_docmodule_comment(of: &mut dyn std::io::Write, s: &str) -> Result<()> {
    write_comment_impl_(of, CommentType::DocModule, s)
}

fn write_comment_impl_(
    of: &mut dyn std::io::Write,
    comment_type: CommentType,
    s: &str,
) -> Result<()> {
    let doc_comment_prefix = match comment_type {
        CommentType::Basic => "//",
        CommentType::Doc => "///",
        CommentType::DocModule => "//!",
    };

    let cur = Cursor::new(s.as_bytes());
    for line_result in cur.lines() {
        let line = line_result?;
        let line = line.trim_end();
        if line.is_empty() {
            writeln!(of, "{doc_comment_prefix}")?;
        } else {
            writeln!(of, "{doc_comment_prefix} {line}")?;
        }
    }
    Ok(())
}

//-------------------------------------------------------------------------------------------------|
