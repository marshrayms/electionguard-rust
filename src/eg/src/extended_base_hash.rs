// Copyright (C) Microsoft Corporation. All rights reserved.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::manual_assert)]
#![allow(unused_imports)] //? TODO: Remove temp development code

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::{
    algebra_utils::to_be_bytes_left_pad,
    eg::Eg,
    errors::EgResult,
    fixed_parameters::FixedParametersTraitExt,
    hash::{HValue, SpecificHValue},
    key::KeyPurpose,
    resource::{
        ElectionDataObjectId as EdoId, ProduceResource, ProduceResourceExt, Resource,
        ResourceFormat, ResourceId, ResourceIdFormat,
    },
    resource_producer::{
        ResourceProductionError, ResourceProductionResult, ResourceSource, ValidReason,
    },
    resource_producer_registry::RPFnRegistration,
    resource_production::RpOp,
    resourceproducer_specific::GatherRPFnRegistrationsFnWrapper,
    serializable::SerializableCanonical,
    standard_parameters::EGDS_V2_1_0_RELEASED_STANDARD_PARAMS_P_LEN_BYTES,
};

//=================================================================================================|

//? TODO Validatable

#[allow(non_camel_case_types)]
pub struct ExtendedBaseHash_tag;

/// `H_E`, the extended base hash.
///
/// EGDS 2.1.0 sec. 3.4.3 eq. 30 pg. 28
#[allow(non_camel_case_types)]
pub type ExtendedBaseHash_H_E = SpecificHValue<ExtendedBaseHash_tag>;

/// The [`ExtendedBaseHash`](crate::extended_base_hash::ExtendedBaseHash), `H_E`.
///
/// EGDS 2.1.0 sec. 3.4.3 eq. 30 pg. 28
#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtendedBaseHash {
    /// Extended base hash.
    h_e: ExtendedBaseHash_H_E,
}

impl ExtendedBaseHash {
    /// `H_E`.
    pub fn h_e(&self) -> &ExtendedBaseHash_H_E {
        &self.h_e
    }

    /// Computes the [`ExtendedBaseHash`], H_E.
    ///
    /// EGDS 2.1.0 sec. 3.2.3 eq. 30 pg. 28
    ///
    /// - `H_E = H(H_B; 0x14, K, K_hat)`
    pub async fn compute(
        produce_resource: &(dyn ProduceResource + Send + Sync + 'static),
    ) -> EgResult<ExtendedBaseHash> {
        let fixed_parameters = produce_resource.fixed_parameters().await?;
        let fixed_parameters = &fixed_parameters;

        let hashes = produce_resource.hashes().await?;
        let h_b = hashes.h_b();

        let jvepk_k = produce_resource
            .joint_vote_encryption_public_key_k()
            .await?;
        let jvepk_k = jvepk_k.as_ref();

        let jbdepk_k_hat = produce_resource
            .joint_ballot_data_encryption_public_key_k_hat()
            .await?;
        let jbdepk_k_hat = jbdepk_k_hat.as_ref();

        // Computation of the extended base hash H_E.
        // EGDS 2.1.0 sec. 3.2.3 eq. 30 pg. 28

        let (p_len_bytes, expected_len) = if fixed_parameters
            .is_not_explicitly_egds_released_specification_standard_parameters()
        {
            let p_len_bytes = fixed_parameters.p_len_bytes();
            let expected_len = 1 + p_len_bytes * 2;
            (p_len_bytes, expected_len)
        } else {
            let p_len_bytes = EGDS_V2_1_0_RELEASED_STANDARD_PARAMS_P_LEN_BYTES;
            let expected_len = 1 + 512 + 512; // EGDS 2.1.0 pg. 74 (30)
            (p_len_bytes, expected_len)
        };

        let mut v = Vec::with_capacity(expected_len);
        v.push(0x14);
        v.extend_from_slice(to_be_bytes_left_pad(jvepk_k, p_len_bytes).as_slice());
        v.extend_from_slice(to_be_bytes_left_pad(jbdepk_k_hat, p_len_bytes).as_slice());

        assert_eq!(v.len(), expected_len);

        let self_ = ExtendedBaseHash {
            h_e: ExtendedBaseHash_H_E::compute_from_eg_h(h_b, &v),
        };

        Ok(self_)
    }
}

impl std::fmt::Display for ExtendedBaseHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ExtendedBaseHash {{ h_e: {} }}", self.h_e)
    }
}

impl std::fmt::Debug for ExtendedBaseHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        std::fmt::Display::fmt(self, f)
    }
}

impl SerializableCanonical for ExtendedBaseHash {}

crate::impl_MayBeValidatableUnsized_for_non_ValidatableUnsized! { ExtendedBaseHash }

crate::impl_Resource_for_simple_ElectionDataObjectId_validated_type! { ExtendedBaseHash, ExtendedBaseHash }

//=================================================================================================|

#[allow(non_upper_case_globals)]
const RID_ExtendedBaseHash: ResourceId = ResourceId::ElectionDataObject(EdoId::ExtendedBaseHash);

#[allow(non_upper_case_globals)]
const RIDFMT_ExtendedBaseHash_ValidatedEdo: ResourceIdFormat = ResourceIdFormat {
    rid: RID_ExtendedBaseHash,
    fmt: ResourceFormat::ValidElectionDataObject,
};

#[allow(non_snake_case)]
fn maybe_produce_ExtendedBaseHash_ValidatedEdo(
    rp_op: &Arc<RpOp>,
) -> Option<ResourceProductionResult> {
    Some(produce_ExtendedBaseHash_ValidatedEdo(rp_op))
}

#[allow(non_snake_case)]
fn produce_ExtendedBaseHash_ValidatedEdo(rp_op: &Arc<RpOp>) -> ResourceProductionResult {
    rp_op.check_ridfmt(&RIDFMT_ExtendedBaseHash_ValidatedEdo)?;

    let extended_base_hash =
        async_global_executor::block_on(ExtendedBaseHash::compute(rp_op.as_ref()))?;

    let arc: Arc<dyn Resource> = Arc::new(extended_base_hash);

    let rpsrc = ResourceSource::Valid(ValidReason::Inherent);
    Ok((arc, rpsrc))
}

//=================================================================================================|

fn gather_rpspecific_registrations(register_fn: &mut dyn FnMut(RPFnRegistration)) {
    register_fn(RPFnRegistration::new_defaultproducer(
        RIDFMT_ExtendedBaseHash_ValidatedEdo,
        Box::new(maybe_produce_ExtendedBaseHash_ValidatedEdo),
    ));
}

inventory::submit! {
    GatherRPFnRegistrationsFnWrapper(gather_rpspecific_registrations)
}

//=================================================================================================|

// Unit tests for the ElectionGuard extended hash.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod t {
    use hex_literal::hex;
    use insta::assert_snapshot;

    use super::*;
    use crate::{eg::Eg, errors::EgResult};

    #[test_log::test]
    fn t1() {
        async_global_executor::block_on(async {
            let eg = Eg::new_with_test_data_generation_and_insecure_deterministic_csprng_seed(
                "eg::extended_base_hash::t::t1",
            );
            let eg = eg.as_ref();

            let extended_base_hash = ExtendedBaseHash::compute(eg).await.unwrap();

            // This hash value has not been computed externally and will need to be modified
            // whenever the example data ElectionManifest changes.
            assert_snapshot!(
                extended_base_hash.h_e,
                @"1AF7B7F385D43E12C9912DF16EB809A3E29225CA7EC771D6E7FA1133EA39D811");

            assert_snapshot!(
                eg.h_p().await.unwrap(),
                @"944286970EAFDB6F347F4EB93B30D48FA3EDCC89BFBAEA6F5AE8F29AFB05DDCE");
        });
    }
}
