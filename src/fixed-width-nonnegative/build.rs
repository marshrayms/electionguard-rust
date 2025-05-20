// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::assertions_on_constants)]
#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

mod build_mod;
use build_mod::*;

use anyhow::{Context, Result, anyhow, bail, ensure};

//=================================================================================================|

fn main() -> Result<()> {
    main2().map_err(|err| {
        use std::io::{BufRead, Cursor, Write};

        let mut cur = Cursor::new(Vec::<u8>::new());
        let _ = writeln!(&mut cur, "ERROR: {err}");
        err.chain().skip(1).for_each(|cause| {
            let _ = writeln!(&mut cur, "because: {}", cause);
        });

        cur.set_position(0);

        #[allow(clippy::manual_flatten)]
        let mut line_n = 0_usize;
        for line_result in cur.lines() {
            match line_result {
                Ok(line) => {
                    let line = line.trim_end();
                    if !line.is_empty() {
                        line_n += 1;
                        println!("cargo:warning=ERROR(line {line_n}): {line}");
                    }
                }
                Err(err) => {
                    line_n += 1;
                    println!("cargo:warning=ERROR(line {line_n}): lines()->Err({err:#?})");
                }
            }
        }

        err
    })
}

//? TODO evaluate whether this approach is worthwhile
use cfg_aliases::cfg_aliases;

#[rustfmt::skip]
fn main2() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    // This allows any code to use `#[cfg(not(not_build_rs))]` to
    // detect when it's being built from `build.rs`.
    println!("cargo:rustc-cfg=not_build_rs");

    // Write the files.
    write_files::write_files()
}
