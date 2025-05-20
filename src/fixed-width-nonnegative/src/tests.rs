// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::const_is_empty)] // This is `cfg(test)` code
#![allow(clippy::unwrap_used)]    // This is `cfg(test)` code
#![allow(clippy::expect_used)]    // This is `cfg(test)` code
#![allow(clippy::panic)]          // This is `cfg(test)` code
#![allow(clippy::manual_assert)]  // This is `cfg(test)` code
#![allow(unused_macros)]
#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(dead_code)] //? TODO: Remove temp development code
#![allow(unused_assignments)] //? TODO: Remove temp development code
#![allow(unused_imports)] //? TODO: Remove temp development code
#![allow(unused_variables)] //? TODO: Remove temp development code
#![allow(unreachable_code)] //? TODO: Remove temp development code
#![allow(non_camel_case_types)] //? TODO: Remove temp development code
#![allow(noop_method_call)] //? TODO: Remove temp development code
#![allow(unused_braces)] //? TODO: Remove temp development code

use anyhow::{anyhow, bail, ensure, Context, Result};
//?use insta::assert_ron_snapshot;
use cfg_if::cfg_if;
use static_assertions::{
    assert_impl_all,
    const_assert,
    const_assert_eq,
};

use crate::primitive_unsigned::*;
use crate::power_of_two::*;

//=================================================================================================|

#[test]
fn t1000() {
    use crate::types::{NumericImpl, NumericImplExt};

    let mut cnt_numimpls = 0_usize;
    for numimpl in crate::types::all_numimpls() {
        //let ns = numimpl.numimpl_struct();
        //assert!(!ns.crate_name.is_empty());

        assert!(!numimpl.crate_name().is_empty());

        // supported_subtypes()
        // repr_is_fixed_size_limbs_array()
        // can_support_multiple_limb_types()
        // supported_limb_types()
        // supported_bits()
        // supports_secure_zeroize()

        cnt_numimpls += 1;

    }
    assert!(0 < cnt_numimpls);
}

#[test]
fn t2000() {
    use crate::types::{TypeInfo, TypeInfoExt};

    let mut cnt_type_infos = 0_usize;
    for type_info in crate::types::all_types() {

        let tis = type_info.typeinfo_struct();
        assert!(!tis.module_name.is_empty());

        // numimpl()

        assert!(!type_info.module_name().is_empty());

        // type_name()

        // limb_type()

        // type_info_fsla_opt()

        // subtype()

        // bits()

        // zeroize()

        cnt_type_infos += 1;

    }
    assert!(0 < cnt_type_infos);
}

#[test]
fn t3000() {
    let mut cnt_numimpls = 0_usize;

    macro_rules! my_callback_for_each_nonnegative_impl {
        ($numimpl_ix:literal, $numimpl_crate_name:literal, $my_param:literal) => {{
            let _ = concat!("vvvvvvvvvvvvvvvvvvvvvvvvvv NUMIMPL[", stringify!($numimpl_ix), "]:", " crate name: ", $numimpl_crate_name);
            let numimpl_ix: usize = $numimpl_ix;

            use crate::types::NumericImpl;
            let numimpl = crate::types::all_numimpls()[$numimpl_ix];
            let crate_name = numimpl.crate_name();
            // println!("\nNUMIMPL[{}] crate name: {crate_name:?}, my_param: {:?}", $numimpl_ix, $my_param);
            assert_eq!($numimpl_crate_name, crate_name);
            assert_eq!($my_param, "my_param_value");

            let my_param: &'static str = $my_param;
            assert_eq!(my_param, "my_param_value");

            cnt_numimpls += 1;
            let _ = concat!("^^^^^^^^^^^^^^^^^^^^^^^^^^ NUMIMPL[", stringify!($numimpl_ix), "]:", " crate name: ", $numimpl_crate_name);
        }};
    }

    for_each_nonnegative_impl!(my_callback_for_each_nonnegative_impl("my_param_value"));

    assert!(0 < cnt_numimpls);
}

#[test]
fn t4000() {
    let mut cnt_types = 0_usize;

    macro_rules! my_callback_for_each_nonnegative_type {
        (
            $type_info_ix:literal,
            $module_name:ident,
            $type_name:ident,
            $bits:literal,
            $my_param:literal
        ) => {
            let s = concat!("vvvvvvvvvvvvvvvvvvvvvvvvvv TYPEINFO[",
                stringify!($type_info_ix),
                "]: module: ", stringify!($module_name),
                ", type: ", stringify!($type_name),
                ", bits: ", stringify!($bits),
            );
            //println!("\n{s}");
            let my_param: &'static str = $my_param;
            assert_eq!(my_param, "my_param_value");
            cnt_types += 1;
            let _ = concat!("^^^^^^^^^^^^^^^^^^^^^^^^^^ TYPEINFO[", stringify!($type_info_ix),
            "]: module: ", stringify!($module_name),
            ", type: ", stringify!($type_name));
        };
    }

    for_each_nonnegative_type!(my_callback_for_each_nonnegative_type("my_param_value"));

    assert!(0 < cnt_types);
}

//=================================================================================================|

macro_rules! cb_for_each_nonnegative_type_make_unique_test_mod {
    (
        $test_module_name:ident,
        $numimpl_ix:literal,
        $type_info_ix:literal,
        $module_name:ident,
        $module_name_fq:path,
        $type_name:ident,
        $type_name_fq:path,
        $bits:tt,
    ) => {
        use crate::types::{NumericImpl, TypeInfo};
        use crate::$module_name::*;

        type Nonnegative = crate::$module_name::$type_name;

        const TEST_MODULE_NAME: &'static str = stringify!($module_name);
        const TEST_TYPE_NAME: &'static str = stringify!($type_name);
        const TEST_BITS: usize = $bits;

        static TEST_TYPE_INFO: &'static (dyn TypeInfo + Sync) = crate::types::all_types()[$type_info_ix];
        static TEST_NUMIPML: &'static (dyn NumericImpl + Sync) = crate::types::all_numimpls()[$numimpl_ix];

        module_name_eq_basicarray_u64! { $module_name => {
            type_name_eq_Nonnegative_4096! { $type_name => {
                include!("tests-basicarray_u64-Nonnegative_4096.inc.rs");
            }}
        }}

        include!("tests-n.inc.rs");
    };
}
for_each_nonnegative_type_make_unique_test_mod!(cb_for_each_nonnegative_type_make_unique_test_mod());

//=================================================================================================|
