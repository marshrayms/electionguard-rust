// Copyright (C) Microsoft Corporation. All rights reserved.

#![allow(clippy::expect_used)] // This is build code
#![allow(clippy::manual_assert)] // This is build code
#![allow(clippy::panic)] // This is build code
#![allow(clippy::unwrap_used)] // This is build code
#![allow(clippy::assertions_on_constants)]
#![allow(non_snake_case)] // This module needs to talk about types in function names
//-

use crate::build_mod::*;
use crate::*;

//=================================================================================================|

fn all_possibly_supported_limb_primitive_types() -> &'static [LimbType] {
    &[LimbType::u32, LimbType::u64]
}

//-------------------------------------------------------------------------------------------------|

#[rustfmt::skip]
fn make_type_author() -> NumericImplAuthor {
    let is_enabled = false;
    #[cfg(feature = "num-bigint")] let is_enabled = true;

    NumericImplAuthor {
        numimpl: NumericImplStruct {
            is_enabled,
            all_possibly_supported_limb_primitive_types: all_possibly_supported_limb_primitive_types(),
            crate_name:                        FwnnAuthorNumBigint::NUMERIC_IMPL_NAME,
            supported_subtypes:                &[ Subtype::Nonnegative ], //Subtype::all(),
            repr_is_fixed_size_limbs_array:    false,
            can_support_multiple_limb_types:   false,
            supported_limb_types:              &[ LimbType::targptr ],
            supported_bits:                    Feature::all_buildrs_knownfeature_bitses_usize(),
            supports_secure_zeroize:           false,
        },
        bx_new_author,
    }
}

inventory::submit! { MakeNumericImplInfo(make_type_author) }

//-------------------------------------------------------------------------------------------------|

pub fn bx_new_author(type_info: &'static TypeInfoStruct) -> Result<Box<dyn Author>> {
    Ok(Box::new(FwnnAuthorNumBigint::try_new(type_info)?))
}

//-------------------------------------------------------------------------------------------------|

struct FwnnAuthorNumBigint {
    type_info: &'static TypeInfoStruct,
}

impl FwnnAuthorNumBigint {
    const NUMERIC_IMPL_NAME: &str = "num-bigint";

    fn try_new(type_info: &'static TypeInfoStruct) -> Result<Self> {
        Ok(FwnnAuthorNumBigint { type_info })
    }
}

impl TypeInfo for FwnnAuthorNumBigint {
    fn typeinfo_struct(&self) -> &TypeInfoStruct {
        self.type_info
    }
}

impl Author for FwnnAuthorNumBigint {
    fn crate_name(&self) -> CowStaticStr {
        Self::NUMERIC_IMPL_NAME.into()
    }

    fn inner_type_module_path_opt(&self) -> Option<TokenStream> {
        Some(quote! { ::num_bigint })
    }

    fn inner_type_turbofish(&self) -> TokenStream {
        self.inner_type_module_fq("BigUint")
    }

    //---------------------------------------------------------------------------------------------|

    fn accumulate_usedecls(&self, uses_set: &mut UsedeclSet) -> Result<()> {
        ensure!(
            !self.typeinfo_struct().zeroize(),
            "No method to securely wipe num-bigint types #118 https://github.com/rust-num/num-bigint/issues/118"
        );
        Ok(())
    }

    //---------------------------------------------------------------------------------------------|

    fn can_const_construct(&self) -> bool {
        // Unfortunately, there's no way to construct a `num_bigint::BinUint` in a const function.
        false
    }

    fn impl_inherent_optconst_fn_zero_body_opt(&self) -> Option<(bool, TokenStream)> {
        let inner_type = self.inner_type_turbofish();
        Some((
            false,
            quote! {
                Self(<#inner_type as ::num_traits::identities::Zero>::zero())
            },
        ))
    }

    fn impl_inherent_optconst_fn_one_body_opt(&self) -> Option<(bool, TokenStream)> {
        let inner_type = self.inner_type_turbofish();
        Some((
            false,
            quote! {
                Self(<#inner_type as ::num_traits::identities::One>::one())
            },
        ))
    }

    fn impl_inherent_optconst_fn_all_ones_body_opt(&self) -> Option<(bool, TokenStream)> {
        let inner_type = self.inner_type_turbofish();
        let bits = self.bits();
        assert_ne!(bits, 0);
        let top_bit = Literal::usize_unsuffixed(bits - 1);
        Some((
            false,
            quote! {
                let mut b = <#inner_type as ::num_traits::identities::Zero>::zero();
                let top_bit = #top_bit;
                b.set_bit(top_bit, true);
                b -= 1_u8;
                b.set_bit(top_bit, true);
                Self(b)
            },
        ))
    }

    //---------------------------------------------------------------------------------------------|

    fn opt_impl_inherent_optconst_fn_from_le_bytes_arr_body(&self) -> Option<(bool, TokenStream)> {
        let inner_type = self.inner_type_turbofish();
        Some((
            false,
            quote! {
                Self(#inner_type::from_bytes_le(&bytes))
            },
        ))
    }

    fn opt_impl_inherent_optconst_fn_from_be_bytes_arr_body(&self) -> Option<(bool, TokenStream)> {
        let inner_type = self.inner_type_turbofish();
        Some((
            false,
            quote! {
                Self(#inner_type::from_bytes_be(&bytes))
            },
        ))
    }

    //------------------------------------------------------ `Author::limbs_lsf_msf_iter_bodies_opt`

    fn limbs_lsf_msf_iter_bodies_opt(&self) -> Option<(TokenStream, TokenStream)> {
        let f = format_ident!(
            "{}",
            match self.limb_type().bits_opt().unwrap() {
                32 => "iter_u32_digits",
                64 => "iter_u64_digits",
                bits => panic!("unexpected limb bits: {bits}"),
            }
        );

        let common = quote! {
            let it_src = self.0.#f();
            let len_src = it_src.len();
            assert!(len_src <= Self::CNT_LIMBS);
            let pad_len = Self::CNT_LIMBS.saturating_sub(len_src);
        };

        let pr = (
            quote! { // least-significant first
                #common
                it_src.chain(std::iter::repeat(Self::LIMB_ZERO).take(pad_len))
            },
            quote! { // most-significant first
                #common
                std::iter::repeat(Self::LIMB_ZERO).take(pad_len).chain(it_src.rev())
            },
        );

        Some(pr)
    }

    //-------------------------------------- `Author::impl_inherent_optconst_fn_into_inner_body_opt`

    fn can_const_into_inner(&self) -> bool {
        // `num-bigint::Biguint` has a has a desctructor, so `into_inner()` can't be `const`.
        false
    }

    //------------------------------------------------- `Author::write_asrefs_and_borrows`

    #[rustfmt::skip]
    fn write_asrefs_and_borrows(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let type_ident = self.type_ident();

        write_tokens!(of, "\n", quote! {
            impl std::convert::AsRef<::num_bigint::BigUint> for #type_ident {
                #[inline]
                fn as_ref(&self) -> &::num_bigint::BigUint {
                    &self.0
                }
            }
        })?;

        write_tokens!(of, "\n", quote! {
            impl std::borrow::Borrow<::num_bigint::BigUint> for #type_ident {
                #[inline]
                fn borrow(&self) -> &::num_bigint::BigUint {
                    &self.0
                }
            }
        })?;

        Ok(())
    }

    //---------------------------------------------------------------- `Author::write_stdcloneclone`

    //---------------------------------------------------------- `Author::write_conversions_to_misc`

    fn write_conversions_to_misc(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let type_ident = self.type_ident();

        write_tokens!(
            of,
            "\n",
            quote! {
                impl ::num_bigint::ToBigInt for #type_ident {
                    #[inline]
                    fn to_bigint(&self) -> Option<::num_bigint::BigInt> {
                        self.0.to_bigint()
                    }
                }
            }
        )?;

        write_tokens!(
            of,
            "\n",
            quote! {
                impl ::num_bigint::ToBigUint for #type_ident {
                    #[inline]
                    fn to_biguint(&self) -> Option<::num_bigint::BigUint> {
                        self.0.to_biguint()
                    }
                }
            }
        )?;

        Ok(())
    }

    //------------------------------------- `Author::write_infallible_convs_from_primitive_unsigned`

    fn inner_type_has_stdconvertfrom_notbigger_primitive_std_u(&self, _src_bits: usize) -> bool {
        true
    }

    //------------------------------------------------------- `Author::write_conversions_from_inner`

    fn write_conversions_from_inner(&self, of: &mut dyn std::io::Write) -> Result<()> {
        let type_ident = self.type_ident();

        write_tokens!(
            of,
            "\n",
            quote! {
                impl std::convert::TryFrom<::num_bigint::BigUint> for #type_ident {
                    type Error = anyhow::Error;
                    fn try_from(src: ::num_bigint::BigUint) -> Result<Self, Self::Error> {
                        ::anyhow::ensure!(((src.bits() + 7)/8) as usize <= Self::BYTES, "Too large");
                        Ok(Self(src))
                    }
                }
            }
        )?;

        write_tokens!(
            of,
            "\n",
            quote! {
                impl std::convert::TryFrom<&::num_bigint::BigUint> for #type_ident {
                    type Error = anyhow::Error;
                    fn try_from(src: &::num_bigint::BigUint) -> Result<Self, Self::Error> {
                        ::anyhow::ensure!(((src.bits() + 7)/8) as usize <= Self::BYTES, "Too large");
                        Ok(Self(src.clone()))
                    }
                }
            }
        )?;

        /* write_tokens!(of, "\n", quote! {
            impl<T: std::borrow::Borrow<::num_bigint::BigUint>> std::convert::TryFrom<T> for #type_ident {
                type Error = anyhow::Error;
                fn try_from(src: T) -> Result<Self, Self::Error> {
                    use std::borrow::Borrow;
                    let src = src.borrow();
                    ::anyhow::ensure!(((src.bits() + 7)/8) as usize <= Self::BYTES, "Too large");
                    Ok(Self(src.clone()))
                }
            }
        })?; */

        Ok(())
    }

    //------------------------------------------------- `Author::write_conversions_from_misc`

    //------------------------------------------------- `Author::write_conversions_from`

    //------------------------------------------------- `Author::write_ops`

    fn opt_impl_inherent_fn_wrapping_add_self_body(&self, rhs: &Ident) -> Option<TokenStream> {
        Some(quote! {
            let mut bu = std::ops::Add::add(&self.0, &#rhs.0);
            bu.set_bit(Self::BITS as u64, false);
            Self(bu)
        })
    }

    fn opt_impl_inherent_fn_wrapping_add_u8_body(&self, rhs: &Ident) -> Option<TokenStream> {
        Some(quote! {
            let mut bu = std::ops::Add::add(&self.0, #rhs);
            bu.set_bit(Self::BITS as u64, false);
            Self(bu)
        })
    }

    //---------------------------------------------------------------------------------------------|
}
