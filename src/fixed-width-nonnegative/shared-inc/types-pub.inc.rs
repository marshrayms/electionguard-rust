// Copyright (C) Microsoft Corporation. All rights reserved.

// NOTE: This file is intended to be include!()ed, in a specific context and not built directly.

pub type CowStaticStr = std::borrow::Cow<'static, str>;

//=================================================================================================|

#[non_exhaustive]
#[derive(Clone, Copy, Debug)]//, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(strum::AsRefStr, strum::Display, strum::VariantArray)]
pub enum NumericEncoding {
    /// Representation is based on some inner that we don't see as a bit pattern.
    Opaque,

    /// 0 <= n <= 2^bits - 1
    NonnegativeBinaryPositional,

    /// -2^(bits - 1) <= n <= 2^(bits - 1) - 1
    SignedTwosComplement,

    /// 0 <= n < modulus (specified elsewhere)
    Montgomery,
}

impl std::cmp::PartialEq for NumericEncoding {
    fn eq(&self, rhs: &Self) -> bool {
        use NumericEncoding::*;
        match (*self, *rhs) {
            (NonnegativeBinaryPositional, NonnegativeBinaryPositional) => true,
            (SignedTwosComplement, SignedTwosComplement) => true,
            (Montgomery, Montgomery) => true,
            _ => false, // In particular, Opaque is not equal to itself
        }
    }
}
impl std::cmp::Eq for NumericEncoding {}

//=================================================================================================|

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(strum::AsRefStr, strum::Display, strum::VariantArray)]
pub enum Subtype {
    /// Number 0 <= N < 2^bits
    Nonnegative,

    /// Number 0 <= N < 2^bits
    Montgomery,

    //Positive,
    //Prime,
    //ModPrime,
}

impl Subtype {
    pub fn all() -> &'static [Subtype] {
        use strum::VariantArray;
        Subtype::VARIANTS
    }

    pub fn numeric_encoding(self) -> NumericEncoding {
        match self {
            Subtype::Nonnegative => NumericEncoding::NonnegativeBinaryPositional,
            Subtype::Montgomery => NumericEncoding::Montgomery,
        }
    }

    pub fn all_bit_patterns_valid(self) -> bool {
        match self {
            Subtype::Nonnegative => true,
            Subtype::Montgomery => false,
        }
    }
}

//=================================================================================================|

#[allow(clippy::derived_hash_with_manual_eq)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum::AsRefStr, strum::Display)]
pub enum LimbType {
    u8,
    u16,
    u32,
    targptr,
    u64,
    u128,
    unknown,
}

impl LimbType {
    /// The number of bits in the limb type, if known.
    pub fn bits_opt(self) -> Option<usize> {
        use LimbType::*;
        let opt_bits = match self {
            u8 => Some(8),
            u16 => Some(16),
            u32 => Some(32),
            targptr => __nonpublic::target_pointer_width_opt(),
            u64 => Some(64),
            u128 => Some(128),
            unknown => None,
        };
        if let Some(b) = opt_bits {
            debug_assert!(0 < b);
            debug_assert!(b < usize::MAX/2);
        }
        opt_bits
    }

    /*
    /// Produces sorting value.
    pub fn sortable_key(self) -> usize {
        let mut sort_key = match self.bits_opt() {
            Some(bits) if bits < usize::MAX/4 => bits*2,
            Some(bits) => (usize::MAX/2).saturating_add(bits),
            None => usize::MAX,
        };
        if self == LimbType::targptr {
            // Sort targptr between 'u32' and 'u64'
            if sort_key < 48*2 {
                sort_key += 1;
            } else {
                sort_key -= 1;
            }
        }
        key
    }

    /// The number of bits in the limb type, or `usize::MAX/2 + adj`.
    ///
    /// This is useful for comparisons.
    pub fn bits_or_adj(self, adj: usize) -> usize {
        debug_assert!(adj < usize::MAX/2);
        if let Some(bits) = self.bits_opt() {
            debug_assert!(bits < usize::MAX/2);
            bits
        } else {
            usize::MAX/2 + adj
        }
    }
    // */
}

//=================================================================================================|

/// Information about a specific numeric implementation, such as `basic-array`.
///
/// Object-safe.
pub trait NumericImpl: Sync {
    // Pure accessors.

    fn numimpl_struct(&self) -> &__nonpublic::NumericImplStruct;

    /// Whether this numeric implementation is enabled by build features.
    fn is_enabled(&self) -> bool {
        self.numimpl_struct().is_enabled
    }

    fn crate_name(&self) -> &'static str {
        self.numimpl_struct().crate_name
    }

    /// All possible module names that could be emitted by this numeric implementation, regardless
    /// of whether they are enabled by build features.
    fn all_possible_module_names(&self) -> &'static [String] {
        self.numimpl_struct().all_possible_module_names()
    }

    fn supported_subtypes(&self) -> &'static [Subtype] {
        self.numimpl_struct().supported_subtypes
    }

    fn repr_is_fixed_size_limbs_array(&self) -> bool {
        self.numimpl_struct().repr_is_fixed_size_limbs_array
    }

    /// This numeric impl may support multiple limb types, so the module name will include the
    /// limb type, like `haclrs_u32`.
    fn can_support_multiple_limb_types(&self) -> bool {
        self.numimpl_struct().can_support_multiple_limb_types
    }

    fn supported_limb_types(&self) -> &'static [LimbType] {
        self.numimpl_struct().supported_limb_types
    }

    fn supported_bits(&self) -> &'static [usize] {
        self.numimpl_struct().supported_bits
    }

    fn supports_secure_zeroize(&self) -> bool {
        self.numimpl_struct().supports_secure_zeroize
    }
}

pub trait NumericImplExt: NumericImpl {
    // Derived functionality.
}

impl<T> NumericImplExt for T where T: NumericImpl { }

/// Info about all configured numeric implementations.
#[cfg(not_build_rs)]
pub const fn all_numimpls() -> &'static [&'static (impl NumericImpl + Sized)] {
    &__nonpublic2::NUMIMPLS_REFS
}

//=================================================================================================|

/// Information about a generated numeric type.
///
/// Object-safe.
pub trait TypeInfo: Sync {
    fn typeinfo_struct(&self) -> &__nonpublic::TypeInfoStruct;

    fn numimpl(&self) -> &'static (dyn NumericImpl + Sync) {
        self.typeinfo_struct().numimpl
    }

    fn module_name(&self) -> &CowStaticStr {
        &self.typeinfo_struct().module_name
    }

    fn module_name_fq(&self) -> &CowStaticStr {
        &self.typeinfo_struct().module_name_fq
    }

    fn type_name(&self) -> &CowStaticStr {
        &self.typeinfo_struct().type_name
    }

    fn type_name_fq(&self) -> &CowStaticStr {
        &self.typeinfo_struct().type_name_fq
    }

    fn limb_type(&self) -> LimbType {
        self.typeinfo_struct().limb_type
    }

    fn type_info_fsla_opt(&self) -> Option<&(dyn TypeInfoFixedSizeLimbArray + Sync)> {
        self.typeinfo_struct().as_dyn_tifsla_opt()
    }

    fn subtype(&self) -> Subtype {
        self.typeinfo_struct().subtype
    }

    fn bits(&self) -> usize {
        self.typeinfo_struct().bits
    }

    fn zeroize(&self) -> bool {
        self.typeinfo_struct().zeroize
    }
}

//-------------------------------------------------------------------------------------------------|

/// Additional functionality that builds on `TypeInfo`.
///
/// Not object-safe.
pub trait TypeInfoExt: TypeInfo {

    /// If `bits` is evenly divisible by `div`, and `div != 0`, the result.
    fn bits_div_exact_opt(&self, div: usize) -> Option<usize> {
        self.bits()
            .checked_div(div)
            .and_then(|floor| (floor * div == self.bits()).then_some(floor))
    }

    /// `ceil(bits/div)`, if div != 0.
    fn bits_div_ceil_opt(&self, div: usize) -> Option<usize> {
        (div != 0).then(|| self.bits().div_ceil(div))
    }

    /// If `bits` is evenly divisible by `8`, the number of bytes required.
    fn bytes_exact_opt(&self) -> Option<usize> {
        self.bits_div_exact_opt(8)
    }

    /// The number of bytes needed to hold `bits`.
    fn bytes_needed(&self) -> usize {
        self.bits().div_ceil(8)
    }

    /// If `limb_bits` is known, returns it.
    fn limb_bits_opt(&self) -> Option<usize> {
        self.limb_type().bits_opt()
    }

    /// If `limb_bits` is known, and it is evenly divisible by `div`, the result.
    fn limb_bits_div_exact_opt(&self, div: usize) -> Option<usize> {
        assert!(div != 0);
        self.limb_bits_opt().and_then(|limb_bits| {
            let floor = limb_bits / div;
            let rem = limb_bits % div;
            (rem == 0).then_some(floor)
        })
    }

    /// If `limb_bits` is known, `ceil(limb_bits/div)`, if div != 0.
    fn limb_bits_div_ceil_opt(&self, div: usize) -> Option<usize> {
        assert!(div != 0);
        self.limb_bits_opt()
            .map(|limb_bits| limb_bits.div_ceil(div))
    }

    /// If `limb_bits` is known, the number of bytes needed to represent a limb, or `None` if `limb_bits` is not evenly divisible by 8.
    fn limb_bytes_exact_opt(&self) -> Option<usize> {
        self.limb_bits_div_exact_opt(8)
    }

    /// If `limb_bits` is known, the number of bytes needed to represent a limb, or `None` if `limb_bits` is not evenly divisible by 8.
    fn limb_bytes_needed_opt(&self) -> Option<usize> {
        self.limb_bits_div_ceil_opt(8)
    }

    /// Returns the number of limbs needed, if the size of a limb in bits is known
    /// and it evenly divides `bits`.
    fn cnt_limbs_exact_opt(&self) -> Option<usize> {
        self.limb_bits_opt()
            .and_then(|limb_bits| self.bits_div_exact_opt(limb_bits))
    }

    /// If `limb_bits` is known, the number of limbs needed.
    fn cnt_limbs_opt(&self) -> Option<usize> {
        self.limb_bits_opt()
            .and_then(|limb_bits| self.bits_div_ceil_opt(limb_bits))
    }

    /// Whether conversion from the other type is lossless and the bit pattern has the same semantics.
    #[rustfmt::skip]
    fn conversion_is_always_lossless_and_same_encoding_from(&self, rhs: &dyn TypeInfoExt) -> bool {
        let lhs = self;

        let lhs_subtype = lhs.subtype();
        let rhs_subtype = rhs.subtype();

        let lhs_numeric_encoding = lhs_subtype.numeric_encoding();
        let rhs_numeric_encoding = rhs_subtype.numeric_encoding();
        if lhs_numeric_encoding != rhs_numeric_encoding { return false; }

        let lhs_all_bit_patterns_valid = lhs_subtype.all_bit_patterns_valid();
        let rhs_all_bit_patterns_valid = rhs_subtype.all_bit_patterns_valid();
        if rhs_all_bit_patterns_valid && !lhs_all_bit_patterns_valid { return false; }

        let lhs_smaller_than_rhs = lhs.bits() < rhs.bits();
        if lhs_smaller_than_rhs { return false; }

        let bits_same = rhs.bits() == lhs.bits();

        use NumericEncoding::*;
        #[allow(clippy::match_like_matches_macro)]
        match ( lhs_numeric_encoding,      rhs_numeric_encoding,        bits_same ) {
            ( NonnegativeBinaryPositional, NonnegativeBinaryPositional,         _ ) => true,
            ( SignedTwosComplement,        SignedTwosComplement,             true ) => true,
            _ => false,
        }
    }
}

impl<T> TypeInfoExt for T where T: TypeInfo + ?Sized {}

//-------------------------------------------------------------------------------------------------|

/// Information about numeric implementation that is known to be based on a fixed-size limb array.
///
/// Object-safe.
pub trait TypeInfoFixedSizeLimbArray: TypeInfo {
    /// Returns the number of limbs, which is known to be known.
    fn cnt_limbs(&self) -> usize {
        // Unwrap() is justified here because `cnt_limbs` is known to be known at this point.
        #[allow(clippy::unwrap_used)]
        self.typeinfo_struct().opt_fsla.unwrap().cnt_limbs
    }

    /// Returns the number of bits in a limb, which is known to be known.
    fn limb_bits(&self) -> usize {
        // Unwrap() is justified here because `limb_bits` is known to be known at this point.
        #[allow(clippy::unwrap_used)]
        self.typeinfo_struct().opt_fsla.unwrap().limb_bits
    }
}

//-------------------------------------------------------------------------------------------------|

/// Info about all configured numeric types.
#[cfg(not_build_rs)]
pub const fn all_types() -> &'static [&'static (impl TypeInfo + Sized)] {
    &__nonpublic2::TYPEINFOS_REFS
}

//=================================================================================================|

/// Modifiers to figure the size of an operand. These are applied in order to the base size.
pub enum OperandSizeModifier {
    /// A fixed number of bits. If this is specified, it might as well be the only one
    Absolute(usize),

    /// An amount multiplied.
    RelativeFactor(isize),

    /// An additional number of bits.
    RelativeAddition(isize),

    /// The number of bits needed to represent 0..size, i.e., `ceil(log2(size))`.
    Ceil_Log2,
}

/// The size of an operand.
pub enum OperandSize {
    /// A fixed number of bits (possibly `0`).
    Absolute(usize),

    /// A number of bits computed from the base size.
    ///
    /// These are applied in order to the base size.
    ///
    /// If this is empty, the size is just the base size.
    Relative(&'static [OperandDef]),
}

pub struct OperandDef {
    /// The operand size in the 'in' direction.
    ///
    /// If the operand is 'out'-only, this will be `OperandSize(Absolute(0))`.
    size_in: OperandSize,

    /// The operand size in the 'out' direction.
    ///
    /// If the operand is 'in'-only, this will be `OperandSize(Absolute(0))`.
    size_out: OperandSize,

    /// The value for the operand is supplied by `self`.
    ///
    /// For example, the LHS of the binary `+=` operator.
    ///
    /// But it could be possible for a numeric type to supply multiple operands, such as a
    /// type representing numbers modulo some prime.
    ///
    is_self: bool,


}

pub enum OperandName {
    Self_,
    Ident(&'static str)
}

/// An operation that a fixed-width-nonnegative type could support.
/// Could be an inherent function, a trait fn, or a standalone fn.
///
/// This describes bit widths in terms of relative constraints.
pub struct NumericOp {
    /// The name is not necessarily unique.
    name: &'static str,

    /// Operand definitions.
    operands: &'static [OperandDef],

    /// Type signature for the function.
    fn_sig: String,

    /// Constraints on operand bit widths.
    /// E.g.
    /// sum(in a, in b, out s, out c), s = max(a, b), c=1
    /// E.g. for sum(in a, in b, in c, in d, out s, out c), s = max(a, b, c, d), c=2
    /// mul_rem(in a, in b, out p, out r) p = r = a + b
    operand_width_constraints: () //? TODO
}

/// An operation that fixed-width-nonnegative types of specific bit sizes could support.
pub struct NumericOpSpecific {
    /// The part described by `NumericOp`.
    numeric_op: &'static NumericOp,
}

//=================================================================================================|
