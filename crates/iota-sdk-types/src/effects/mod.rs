// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod v1;

use enum_dispatch::enum_dispatch;
pub use v1::{
    ChangedObject, ObjectIn, ObjectOut, TransactionEffectsV1, UnchangedSharedKind,
    UnchangedSharedObject,
};

use crate::{
    Digest, EpochId, GasCostSummary, ObjectId, ObjectReference, Owner, Version,
    execution_status::ExecutionStatus,
};

/// The output or effects of executing a transaction
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// transaction-effects =  %x00 effects-v1
///                     =/ %x01 effects-v2
/// ```
#[enum_dispatch(TransactionEffectsAPI, TransactionEffectsAPIForTesting)]
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(
    feature = "schemars",
    derive(schemars::JsonSchema),
    schemars(tag = "version")
)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[non_exhaustive]
pub enum TransactionEffects {
    #[cfg_attr(feature = "schemars", schemars(rename = "1"))]
    V1(Box<TransactionEffectsV1>),
}

impl TransactionEffects {
    crate::def_is!(V1);

    pub fn as_v1(&self) -> &TransactionEffectsV1 {
        let Self::V1(effects) = self;
        effects
    }

    pub fn into_v1(self) -> TransactionEffectsV1 {
        let Self::V1(effects) = self;
        *effects
    }

    /// Return the status of the transaction.
    pub fn status(&self) -> &ExecutionStatus {
        match self {
            TransactionEffects::V1(e) => e.status(),
        }
    }

    /// Return the epoch in which this transaction was executed.
    pub fn epoch(&self) -> u64 {
        match self {
            TransactionEffects::V1(e) => e.epoch(),
        }
    }

    /// Return the gas cost summary of the transaction.
    pub fn gas_summary(&self) -> &crate::gas::GasCostSummary {
        match self {
            TransactionEffects::V1(e) => e.gas_summary(),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod serialization {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{TransactionEffects, TransactionEffectsV1};

    #[derive(serde::Serialize)]
    #[serde(tag = "version")]
    enum ReadableEffectsRef<'a> {
        #[serde(rename = "1")]
        V1(&'a TransactionEffectsV1),
    }

    #[derive(serde::Deserialize)]
    #[serde(tag = "version")]
    pub enum ReadableEffects {
        #[serde(rename = "1")]
        V1(Box<TransactionEffectsV1>),
    }

    #[derive(serde::Serialize)]
    enum BinaryEffectsRef<'a> {
        V1(&'a TransactionEffectsV1),
    }

    #[derive(serde::Deserialize)]
    pub enum BinaryEffects {
        V1(Box<TransactionEffectsV1>),
    }

    impl Serialize for TransactionEffects {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                let readable = match self {
                    TransactionEffects::V1(fx) => ReadableEffectsRef::V1(fx),
                };
                readable.serialize(serializer)
            } else {
                let binary = match self {
                    TransactionEffects::V1(fx) => BinaryEffectsRef::V1(fx),
                };
                binary.serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for TransactionEffects {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                ReadableEffects::deserialize(deserializer).map(|readable| match readable {
                    ReadableEffects::V1(fx) => Self::V1(fx),
                })
            } else {
                BinaryEffects::deserialize(deserializer).map(|binary| match binary {
                    BinaryEffects::V1(fx) => Self::V1(fx),
                })
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use base64ct::{Base64, Encoding};
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen_test::wasm_bindgen_test as test;

        use super::TransactionEffects;

        #[test]
        fn effects_fixtures() {
            // The files contain the bas64 encoded raw effects of transactions
            const GENESIS_EFFECTS: &str = include_str!("fixtures/genesis-transaction-effects");
            const SPONSOR_TX_EFFECTS: &str = include_str!("fixtures/sponsor-tx-effects");

            for fixture in [GENESIS_EFFECTS, SPONSOR_TX_EFFECTS] {
                let fixture = Base64::decode_vec(fixture.trim()).unwrap();
                let fx: TransactionEffects = bcs::from_bytes(&fixture).unwrap();
                assert_eq!(bcs::to_bytes(&fx).unwrap(), fixture);

                let json = serde_json::to_string_pretty(&fx).unwrap();
                println!("{json}");
                assert_eq!(fx, serde_json::from_str(&json).unwrap());
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum InputSharedObject {
    Mutate(ObjectReference),
    ReadOnly(ObjectReference),
    ReadDeleted(ObjectId, Version),
    MutateDeleted(ObjectId, Version),
    Cancelled(ObjectId, Version),
}

impl InputSharedObject {
    pub fn id_and_version(&self) -> (ObjectId, Version) {
        let (object_id, version, ..) = self.object_ref().into_parts();
        (object_id, version)
    }

    pub fn object_ref(&self) -> ObjectReference {
        match self {
            InputSharedObject::Mutate(oref) | InputSharedObject::ReadOnly(oref) => *oref,
            InputSharedObject::ReadDeleted(id, version)
            | InputSharedObject::MutateDeleted(id, version) => {
                ObjectReference::new(*id, *version, Digest::OBJECT_DELETED)
            }
            InputSharedObject::Cancelled(id, version) => {
                ObjectReference::new(*id, *version, Digest::OBJECT_CANCELLED)
            }
        }
    }
}

/// Defines what happened to an ObjectId during execution
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// id-operation =  id-operation-none
///              =/ id-operation-created
///              =/ id-operation-deleted
///
/// id-operation-none       = %x00
/// id-operation-created    = %x01
/// id-operation-deleted    = %x02
/// ```
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[non_exhaustive]
pub enum IdOperation {
    None,
    Created,
    Deleted,
}

impl IdOperation {
    crate::def_is!(None, Created, Deleted);
}

#[derive(Clone)]
pub struct ObjectChange {
    pub id: ObjectId,
    pub input_version: Option<Version>,
    pub input_digest: Option<Digest>,
    pub output_version: Option<Version>,
    pub output_digest: Option<Digest>,
    pub id_operation: IdOperation,
}

#[enum_dispatch]
pub trait TransactionEffectsAPI {
    fn status(&self) -> &ExecutionStatus;
    fn into_status(self) -> ExecutionStatus;
    fn epoch(&self) -> EpochId;
    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)>;
    /// The version assigned to all output objects (apart from packages).
    fn lamport_version(&self) -> Version;
    fn old_object_metadata(&self) -> Vec<(ObjectReference, Owner)>;
    fn input_shared_objects(&self) -> Vec<InputSharedObject>;
    fn created(&self) -> Vec<(ObjectReference, Owner)>;
    fn mutated(&self) -> Vec<(ObjectReference, Owner)>;
    fn unwrapped(&self) -> Vec<(ObjectReference, Owner)>;
    fn deleted(&self) -> Vec<ObjectReference>;
    fn unwrapped_then_deleted(&self) -> Vec<ObjectReference>;
    fn wrapped(&self) -> Vec<ObjectReference>;
    fn object_changes(&self) -> Vec<ObjectChange>;
    // TODO: We should consider having this function to return Option.
    // When the gas object is not available (i.e. system transaction), we currently
    // return dummy object ref and owner. This is not ideal.
    fn gas_object(&self) -> (ObjectReference, Owner);
    fn events_digest(&self) -> Option<&Digest>;
    fn dependencies(&self) -> &[Digest];
    fn transaction_digest(&self) -> &Digest;
    fn gas_cost_summary(&self) -> &GasCostSummary;
    fn deleted_mutably_accessed_shared_objects(&self) -> Vec<ObjectId> {
        self.input_shared_objects()
            .into_iter()
            .filter_map(|kind| match kind {
                InputSharedObject::MutateDeleted(id, _) => Some(id),
                InputSharedObject::Mutate(..)
                | InputSharedObject::ReadOnly(..)
                | InputSharedObject::ReadDeleted(..)
                | InputSharedObject::Cancelled(..) => None,
            })
            .collect()
    }
    fn unchanged_shared_objects(&self) -> Vec<(ObjectId, UnchangedSharedKind)>;
}

#[enum_dispatch]
pub trait TransactionEffectsAPIForTesting: TransactionEffectsAPI {
    fn status_mut_for_testing(&mut self) -> &mut ExecutionStatus;
    fn gas_cost_summary_mut_for_testing(&mut self) -> &mut GasCostSummary;
    fn transaction_digest_mut_for_testing(&mut self) -> &mut Digest;
    fn dependencies_mut_for_testing(&mut self) -> &mut Vec<Digest>;
    fn unsafe_add_input_shared_object_for_testing(&mut self, kind: InputSharedObject);
    // Adding an old version of a live object.
    fn unsafe_add_deleted_live_object_for_testing(&mut self, object_ref: ObjectReference);
    // Adding a tombstone for a deleted object.
    fn unsafe_add_object_tombstone_for_testing(&mut self, object_ref: ObjectReference);
}
