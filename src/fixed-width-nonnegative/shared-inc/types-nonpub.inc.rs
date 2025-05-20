// Copyright (C) Microsoft Corporation. All rights reserved.

// NOTE: This file is intended to be include!()ed, in a specific context and not built directly.

/// These types are expected to change, please don't write code that relies on them.
/// Use the public traits instead.
mod __nonpublic {
    use super::*;

    //=============================================================================================|

    /// When built as part of `build.rs`.
    #[cfg(not(not_build_rs))]
    #[inline]
    pub fn target_pointer_width_opt() -> Option<usize> {
        crate::buildrs_target_pointer_width_opt()
    }

    /// When building regular lib target.
    #[cfg(not_build_rs)]
    #[inline]
    pub fn target_pointer_width_opt() -> Option<usize> {
        let tpw_bytes = ::std::mem::size_of::<*const ::std::ffi::c_void>();
        Some(tpw_bytes*8)
    }

    //=============================================================================================|

    /// Struct holding information about a numeric implementation.
    /// This is expected to evolve over time - use the public traits to access this information
    /// outside the build system.
    #[derive(Clone, Debug, PartialEq, PartialOrd, Hash)]
    pub struct NumericImplStruct {
        pub is_enabled: bool,
        pub all_possibly_supported_limb_primitive_types: &'static [LimbType],
        //?pub all_possible_module_names: &'static [String],
        pub crate_name: &'static str,
        pub supported_subtypes: &'static [Subtype],
        pub repr_is_fixed_size_limbs_array: bool,
        pub can_support_multiple_limb_types: bool,
        pub supported_limb_types: &'static [LimbType],
        pub supported_bits: &'static [usize],
        pub supports_secure_zeroize: bool,
    }

    impl super::NumericImpl for NumericImplStruct {
        fn numimpl_struct(&self) -> &NumericImplStruct {
            self
        }
    }

    //=============================================================================================|

    #[non_exhaustive]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FixedSizeLimbArr {
        pub cnt_limbs: usize,
        pub limb_bits: usize,
    }

    //=============================================================================================|

    /// Struct holding information about a generated type.
    /// This is expected to evolve over time - use the public traits to access this information
    /// outside the build system.
    #[derive(Debug, PartialEq, PartialOrd, Hash)]
    pub struct TypeInfoStruct {
        pub numimpl: &'static NumericImplStruct,
        pub module_name: CowStaticStr,
        pub module_name_fq: CowStaticStr,
        pub type_name: CowStaticStr,
        pub type_name_fq: CowStaticStr,
        pub limb_type: LimbType,
        pub opt_fsla: Option<FixedSizeLimbArr>,
        pub subtype: Subtype,
        pub bits: usize,
        pub zeroize: bool,
    }

    impl TypeInfoStruct {
        pub fn as_dyn_tifsla_opt(&self) -> Option<&(dyn TypeInfoFixedSizeLimbArray + Sync)> {
            self.opt_fsla.map(|_| self as &(dyn TypeInfoFixedSizeLimbArray + Sync))
        }
    }

    impl TypeInfo for TypeInfoStruct {
        fn typeinfo_struct(&self) -> &TypeInfoStruct { self }
    }

    impl TypeInfoFixedSizeLimbArray for TypeInfoStruct { }
}
