// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::{Deref, DerefMut};

use crate::{
    Address, Digest, EpochId, GasCostSummary, IdOperation, ObjectId, ObjectReference, Version,
    effects::{
        InputSharedObject, ObjectChange, TransactionEffectsAPI, TransactionEffectsAPIForTesting,
    },
    execution_status::ExecutionStatus,
    object::{OBJECT_START_VERSION, Owner},
};

/// Version 1 of TransactionEffects
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// effects-v1 = execution-status
///              u64                                ; epoch
///              gas-cost-summary
///              digest                             ; transaction digest
///              (option u32)                       ; gas object index
///              (option digest)                    ; events digest
///              (vector digest)                    ; list of transaction dependencies
///              u64                                ; lamport version
///              (vector changed-object)
///              (vector unchanged-shared-object)
///              (option digest)                    ; auxiliary data digest
/// ```
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct TransactionEffectsV1 {
    /// The status of the execution
    #[cfg_attr(feature = "schemars", schemars(flatten))]
    pub status: ExecutionStatus,
    /// The epoch when this transaction was executed.
    #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
    pub epoch: EpochId,
    /// The gas used by this transaction
    pub gas_used: GasCostSummary,
    /// The transaction digest
    pub transaction_digest: Digest,
    /// The updated gas object reference, as an index into the `changed_objects`
    /// vector. Having a dedicated field for convenient access.
    /// System transaction that don't require gas will leave this as None.
    pub gas_object_index: Option<u32>,
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    pub events_digest: Option<Digest>,
    /// The set of transaction digests this transaction depends on.
    #[cfg_attr(feature = "proptest", any(proptest::collection::size_range(0..=5).lift()))]
    pub dependencies: Vec<Digest>,
    /// The version number of all the written Move objects by this transaction.
    #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
    pub lamport_version: Version,
    /// Objects whose state are changed in the object store.
    #[cfg_attr(feature = "proptest", any(proptest::collection::size_range(0..=2).lift()))]
    pub changed_objects: Vec<ChangedObject>,
    /// Shared objects that are not mutated in this transaction. Unlike owned
    /// objects, read-only shared objects' version are not committed in the
    /// transaction, and in order for a node to catch up and execute it
    /// without consensus sequencing, the version needs to be committed in
    /// the effects.
    #[cfg_attr(feature = "proptest", any(proptest::collection::size_range(0..=2).lift()))]
    pub unchanged_shared_objects: Vec<UnchangedSharedObject>,
    /// Auxiliary data that are not protocol-critical, generated as part of the
    /// effects but are stored separately. Storing it separately allows us
    /// to avoid bloating the effects with data that are not critical.
    /// It also provides more flexibility on the format and type of the data.
    pub auxiliary_data_digest: Option<Digest>,
}

impl TransactionEffectsV1 {
    /// The gas used in this transaction.
    pub fn gas_summary(&self) -> &GasCostSummary {
        &self.gas_used
    }
}

impl<T: TransactionEffectsAPI> TransactionEffectsAPI for Box<T> {
    fn status(&self) -> &ExecutionStatus {
        self.deref().status()
    }

    fn into_status(self) -> ExecutionStatus {
        (*self).into_status()
    }

    fn epoch(&self) -> EpochId {
        self.deref().epoch()
    }

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)> {
        self.deref().modified_at_versions()
    }

    fn lamport_version(&self) -> Version {
        self.deref().lamport_version()
    }

    fn old_object_metadata(&self) -> Vec<(ObjectReference, Owner)> {
        self.deref().old_object_metadata()
    }

    fn input_shared_objects(&self) -> Vec<InputSharedObject> {
        self.deref().input_shared_objects()
    }

    fn created(&self) -> Vec<(ObjectReference, Owner)> {
        self.deref().created()
    }

    fn mutated(&self) -> Vec<(ObjectReference, Owner)> {
        self.deref().mutated()
    }

    fn unwrapped(&self) -> Vec<(ObjectReference, Owner)> {
        self.deref().unwrapped()
    }

    fn deleted(&self) -> Vec<ObjectReference> {
        self.deref().deleted()
    }

    fn unwrapped_then_deleted(&self) -> Vec<ObjectReference> {
        self.deref().unwrapped_then_deleted()
    }

    fn wrapped(&self) -> Vec<ObjectReference> {
        self.deref().wrapped()
    }

    fn object_changes(&self) -> Vec<ObjectChange> {
        self.deref().object_changes()
    }

    fn gas_object(&self) -> (ObjectReference, Owner) {
        self.deref().gas_object()
    }

    fn events_digest(&self) -> Option<&Digest> {
        self.deref().events_digest()
    }

    fn dependencies(&self) -> &[Digest] {
        self.deref().dependencies()
    }

    fn transaction_digest(&self) -> &Digest {
        self.deref().transaction_digest()
    }

    fn gas_cost_summary(&self) -> &GasCostSummary {
        self.deref().gas_cost_summary()
    }

    fn unchanged_shared_objects(&self) -> Vec<(ObjectId, UnchangedSharedKind)> {
        self.deref().unchanged_shared_objects()
    }
}

impl<T: TransactionEffectsAPIForTesting> TransactionEffectsAPIForTesting for Box<T> {
    fn status_mut_for_testing(&mut self) -> &mut ExecutionStatus {
        self.deref_mut().status_mut_for_testing()
    }

    fn gas_cost_summary_mut_for_testing(&mut self) -> &mut GasCostSummary {
        self.deref_mut().gas_cost_summary_mut_for_testing()
    }

    fn transaction_digest_mut_for_testing(&mut self) -> &mut Digest {
        self.deref_mut().transaction_digest_mut_for_testing()
    }

    fn dependencies_mut_for_testing(&mut self) -> &mut Vec<Digest> {
        self.deref_mut().dependencies_mut_for_testing()
    }

    fn unsafe_add_input_shared_object_for_testing(&mut self, kind: InputSharedObject) {
        self.deref_mut()
            .unsafe_add_input_shared_object_for_testing(kind);
    }

    fn unsafe_add_deleted_live_object_for_testing(&mut self, object_ref: ObjectReference) {
        self.deref_mut()
            .unsafe_add_deleted_live_object_for_testing(object_ref);
    }

    fn unsafe_add_object_tombstone_for_testing(&mut self, object_ref: ObjectReference) {
        self.deref_mut()
            .unsafe_add_object_tombstone_for_testing(object_ref);
    }
}

impl TransactionEffectsAPI for TransactionEffectsV1 {
    fn status(&self) -> &ExecutionStatus {
        &self.status
    }

    fn into_status(self) -> ExecutionStatus {
        self.status
    }

    fn epoch(&self) -> EpochId {
        self.epoch
    }

    fn modified_at_versions(&self) -> Vec<(ObjectId, Version)> {
        self.changed_objects
            .iter()
            .filter_map(|change| {
                if let ObjectIn::Data { version, .. } = &change.input_state {
                    Some((change.object_id, *version))
                } else {
                    None
                }
            })
            .collect()
    }

    fn lamport_version(&self) -> Version {
        self.lamport_version
    }

    fn old_object_metadata(&self) -> Vec<(ObjectReference, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(|change| {
                if let ObjectIn::Data {
                    version,
                    digest,
                    owner,
                } = change.input_state
                {
                    Some((
                        ObjectReference::new(change.object_id, version, digest),
                        owner,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    fn input_shared_objects(&self) -> Vec<InputSharedObject> {
        self.changed_objects
            .iter()
            .filter_map(|changed| {
                if let ObjectIn::Data {
                    version,
                    digest,
                    owner: Owner::Shared { .. },
                } = changed.input_state
                {
                    Some(InputSharedObject::Mutate(ObjectReference::new(
                        changed.object_id,
                        version,
                        digest,
                    )))
                } else {
                    None
                }
            })
            .chain(self.unchanged_shared_objects.iter().filter_map(
                |unchanged| match unchanged.kind {
                    UnchangedSharedKind::ReadOnlyRoot { version, digest } => {
                        Some(InputSharedObject::ReadOnly(ObjectReference::new(
                            unchanged.object_id,
                            version,
                            digest,
                        )))
                    }
                    UnchangedSharedKind::MutateDeleted { version } => Some(
                        InputSharedObject::MutateDeleted(unchanged.object_id, version),
                    ),
                    UnchangedSharedKind::ReadDeleted { version } => {
                        Some(InputSharedObject::ReadDeleted(unchanged.object_id, version))
                    }
                    UnchangedSharedKind::Cancelled { version } => {
                        Some(InputSharedObject::Cancelled(unchanged.object_id, version))
                    }
                    // We can not expose the per epoch config object as input shared object,
                    // since it does not require sequencing, and hence shall not be considered
                    // as a normal input shared object.
                    UnchangedSharedKind::PerEpochConfig => None,
                },
            ))
            .collect()
    }

    fn created(&self) -> Vec<(ObjectReference, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(|changed| {
                match (
                    &changed.input_state,
                    &changed.output_state,
                    &changed.id_operation,
                ) {
                    (
                        ObjectIn::Missing,
                        ObjectOut::ObjectWrite { digest, owner },
                        IdOperation::Created,
                    ) => Some((
                        ObjectReference::new(changed.object_id, self.lamport_version, *digest),
                        *owner,
                    )),
                    (
                        ObjectIn::Missing,
                        ObjectOut::PackageWrite { version, digest },
                        IdOperation::Created,
                    ) => Some((
                        ObjectReference::new(changed.object_id, *version, *digest),
                        Owner::Immutable,
                    )),
                    _ => None,
                }
            })
            .collect()
    }

    fn mutated(&self) -> Vec<(ObjectReference, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(
                |changed| match (&changed.input_state, &changed.output_state) {
                    (ObjectIn::Data { .. }, ObjectOut::ObjectWrite { digest, owner }) => Some((
                        ObjectReference::new(changed.object_id, self.lamport_version, *digest),
                        *owner,
                    )),
                    (ObjectIn::Data { .. }, ObjectOut::PackageWrite { version, digest }) => Some((
                        ObjectReference::new(changed.object_id, *version, *digest),
                        Owner::Immutable,
                    )),
                    _ => None,
                },
            )
            .collect()
    }

    fn unwrapped(&self) -> Vec<(ObjectReference, Owner)> {
        self.changed_objects
            .iter()
            .filter_map(|changed| {
                match (
                    &changed.input_state,
                    &changed.output_state,
                    &changed.id_operation,
                ) {
                    (
                        ObjectIn::Missing,
                        ObjectOut::ObjectWrite { digest, owner },
                        IdOperation::None,
                    ) => Some((
                        ObjectReference::new(changed.object_id, self.lamport_version, *digest),
                        *owner,
                    )),
                    _ => None,
                }
            })
            .collect()
    }

    fn deleted(&self) -> Vec<ObjectReference> {
        self.changed_objects
            .iter()
            .filter_map(|changed| {
                match (
                    &changed.input_state,
                    &changed.output_state,
                    &changed.id_operation,
                ) {
                    (ObjectIn::Data { .. }, ObjectOut::Missing, IdOperation::Deleted) => {
                        Some(ObjectReference::new(
                            changed.object_id,
                            self.lamport_version,
                            Digest::OBJECT_DELETED,
                        ))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn unwrapped_then_deleted(&self) -> Vec<ObjectReference> {
        self.changed_objects
            .iter()
            .filter_map(|changed| {
                match (
                    &changed.input_state,
                    &changed.output_state,
                    &changed.id_operation,
                ) {
                    (ObjectIn::Missing, ObjectOut::Missing, IdOperation::Deleted) => {
                        Some(ObjectReference::new(
                            changed.object_id,
                            self.lamport_version,
                            Digest::OBJECT_DELETED,
                        ))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn wrapped(&self) -> Vec<ObjectReference> {
        self.changed_objects
            .iter()
            .filter_map(|changed| {
                match (
                    &changed.input_state,
                    &changed.output_state,
                    &changed.id_operation,
                ) {
                    (ObjectIn::Data { .. }, ObjectOut::Missing, IdOperation::None) => {
                        Some(ObjectReference::new(
                            changed.object_id,
                            self.lamport_version,
                            Digest::OBJECT_WRAPPED,
                        ))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn object_changes(&self) -> Vec<ObjectChange> {
        self.changed_objects
            .iter()
            .map(|changed| {
                let input_version_digest = match &changed.input_state {
                    ObjectIn::Missing => None,
                    ObjectIn::Data {
                        version, digest, ..
                    } => Some((version, digest)),
                };

                let output_version_digest = match &changed.output_state {
                    ObjectOut::Missing => None,
                    ObjectOut::ObjectWrite { digest, .. } => Some((&self.lamport_version, digest)),
                    ObjectOut::PackageWrite { version, digest } => Some((version, digest)),
                };

                ObjectChange {
                    id: changed.object_id,
                    input_version: input_version_digest.map(|k| *k.0),
                    input_digest: input_version_digest.map(|k| *k.1),
                    output_version: output_version_digest.map(|k| *k.0),
                    output_digest: output_version_digest.map(|k| *k.1),
                    id_operation: changed.id_operation,
                }
            })
            .collect()
    }

    fn gas_object(&self) -> (ObjectReference, Owner) {
        if let Some(gas_object_index) = self.gas_object_index {
            let changed = &self.changed_objects[gas_object_index as usize];
            match changed.output_state {
                ObjectOut::ObjectWrite { digest, owner } => (
                    ObjectReference::new(changed.object_id, self.lamport_version, digest),
                    owner,
                ),
                _ => panic!("Gas object must be an ObjectWrite in changed_objects"),
            }
        } else {
            (
                ObjectReference::new(ObjectId::ZERO, Version::default(), Digest::MIN),
                Owner::Address(Address::ZERO),
            )
        }
    }

    fn events_digest(&self) -> Option<&Digest> {
        self.events_digest.as_ref()
    }

    fn dependencies(&self) -> &[Digest] {
        &self.dependencies
    }

    fn transaction_digest(&self) -> &Digest {
        &self.transaction_digest
    }

    fn gas_cost_summary(&self) -> &GasCostSummary {
        &self.gas_used
    }

    fn unchanged_shared_objects(&self) -> Vec<(ObjectId, UnchangedSharedKind)> {
        self.unchanged_shared_objects
            .iter()
            .map(|unchanged| (unchanged.object_id, unchanged.kind.clone()))
            .collect()
    }
}

impl TransactionEffectsAPIForTesting for TransactionEffectsV1 {
    fn status_mut_for_testing(&mut self) -> &mut ExecutionStatus {
        &mut self.status
    }

    fn gas_cost_summary_mut_for_testing(&mut self) -> &mut GasCostSummary {
        &mut self.gas_used
    }

    fn transaction_digest_mut_for_testing(&mut self) -> &mut Digest {
        &mut self.transaction_digest
    }

    fn dependencies_mut_for_testing(&mut self) -> &mut Vec<Digest> {
        &mut self.dependencies
    }

    fn unsafe_add_input_shared_object_for_testing(&mut self, kind: InputSharedObject) {
        match kind {
            InputSharedObject::Mutate(object_ref) => {
                let (object_id, version, digest) = object_ref.into_parts();
                self.changed_objects.push(ChangedObject {
                    object_id,
                    input_state: ObjectIn::Data {
                        version,
                        digest,
                        owner: Owner::Shared(OBJECT_START_VERSION),
                    },
                    output_state: ObjectOut::ObjectWrite {
                        digest,
                        owner: Owner::Shared(version),
                    },
                    id_operation: IdOperation::None,
                })
            }
            InputSharedObject::ReadOnly(object_ref) => {
                let (object_id, version, digest) = object_ref.into_parts();
                self.unchanged_shared_objects.push(UnchangedSharedObject {
                    object_id,
                    kind: UnchangedSharedKind::ReadOnlyRoot { version, digest },
                })
            }
            InputSharedObject::ReadDeleted(object_id, version) => {
                self.unchanged_shared_objects.push(UnchangedSharedObject {
                    object_id,
                    kind: UnchangedSharedKind::ReadDeleted { version },
                })
            }
            InputSharedObject::MutateDeleted(object_id, version) => {
                self.unchanged_shared_objects.push(UnchangedSharedObject {
                    object_id,
                    kind: UnchangedSharedKind::MutateDeleted { version },
                })
            }
            InputSharedObject::Cancelled(object_id, version) => {
                self.unchanged_shared_objects.push(UnchangedSharedObject {
                    object_id,
                    kind: UnchangedSharedKind::Cancelled { version },
                })
            }
        }
    }

    fn unsafe_add_deleted_live_object_for_testing(&mut self, object_ref: ObjectReference) {
        let (object_id, version, digest) = object_ref.into_parts();
        self.changed_objects.push(ChangedObject {
            object_id,
            input_state: ObjectIn::Data {
                version,
                digest,
                owner: Owner::Address(Address::ZERO),
            },
            output_state: ObjectOut::ObjectWrite {
                digest,
                owner: Owner::Address(Address::ZERO),
            },
            id_operation: IdOperation::None,
        })
    }

    fn unsafe_add_object_tombstone_for_testing(&mut self, object_ref: ObjectReference) {
        let (object_id, version, digest) = object_ref.into_parts();
        self.changed_objects.push(ChangedObject {
            object_id,
            input_state: ObjectIn::Data {
                version,
                digest,
                owner: Owner::Address(Address::ZERO),
            },
            output_state: ObjectOut::Missing,
            id_operation: IdOperation::Deleted,
        })
    }
}

/// Input/output state of an object that was changed during execution
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// changed-object = object-id object-in object-out id-operation
/// ```
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct ChangedObject {
    /// Id of the object
    pub object_id: ObjectId,
    /// State of the object in the store prior to this transaction.
    pub input_state: ObjectIn,
    /// State of the object in the store after this transaction.
    pub output_state: ObjectOut,
    /// Whether this object ID is created or deleted in this transaction.
    /// This information isn't required by the protocol but is useful for
    /// providing more detailed semantics on object changes.
    pub id_operation: IdOperation,
}

/// A shared object that wasn't changed during execution
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// unchanged-shared-object = object-id unchanged-shared-object-kind
/// ```
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct UnchangedSharedObject {
    pub object_id: ObjectId,
    pub kind: UnchangedSharedKind,
}

/// Type of unchanged shared object
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// unchanged-shared-object-kind =  read-only-root
///                              =/ mutate-deleted
///                              =/ read-deleted
///                              =/ cancelled
///                              =/ per-epoch-config
///
/// read-only-root      = %x00 u64 digest
/// mutate-deleted      = %x01 u64
/// read-deleted        = %x02 u64
/// cancelled           = %x03 u64
/// per-epoch-config    = %x04
/// ```
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(
    feature = "schemars",
    derive(schemars::JsonSchema),
    schemars(tag = "kind", rename_all = "snake_case")
)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[non_exhaustive]
pub enum UnchangedSharedKind {
    /// Read-only shared objects from the input. We don't really need
    /// ObjectDigest for protocol correctness, but it will make it easier to
    /// verify untrusted read.
    ReadOnlyRoot {
        #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
        version: Version,
        digest: Digest,
    },
    /// Deleted shared objects that appear mutably/owned in the input.
    MutateDeleted {
        #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
        version: Version,
    },
    /// Deleted shared objects that appear as read-only in the input.
    ReadDeleted {
        #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
        version: Version,
    },
    /// Shared objects in cancelled transaction. The sequence number embed
    /// cancellation reason.
    Cancelled {
        #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
        version: Version,
    },
    /// Read of a per-epoch config object that should remain the same during an
    /// epoch.
    PerEpochConfig,
}

impl UnchangedSharedKind {
    crate::def_is!(
        ReadOnlyRoot,
        MutateDeleted,
        ReadDeleted,
        Cancelled,
        PerEpochConfig
    );
}

/// State of an object prior to execution
///
/// If an object exists (at root-level) in the store prior to this transaction,
/// it should be Data, otherwise it's Missing, e.g. wrapped objects should be
/// Missing.
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// object-in = object-in-missing / object-in-data
///
/// object-in-missing = %x00
/// object-in-data    = %x01 u64 digest owner
/// ```
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(
    feature = "schemars",
    derive(schemars::JsonSchema),
    schemars(tag = "state", rename_all = "snake_case")
)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[non_exhaustive]
pub enum ObjectIn {
    Missing,
    /// The old version, digest and owner.
    Data {
        #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
        version: Version,
        digest: Digest,
        owner: Owner,
    },
}

impl ObjectIn {
    crate::def_is!(Missing, Data);

    pub fn version_opt(&self) -> Option<Version> {
        if let Self::Data { version, .. } = self {
            Some(*version)
        } else {
            None
        }
    }

    pub fn version(&self) -> Version {
        self.version_opt().expect("object does not exist")
    }

    pub fn digest_opt(&self) -> Option<Digest> {
        if let Self::Data { digest, .. } = self {
            Some(*digest)
        } else {
            None
        }
    }

    pub fn digest(&self) -> Digest {
        self.digest_opt().expect("object does not exist")
    }

    pub fn owner_opt(&self) -> Option<Owner> {
        if let Self::Data { owner, .. } = self {
            Some(*owner)
        } else {
            None
        }
    }

    pub fn owner(&self) -> Owner {
        self.owner_opt().expect("object does not exist")
    }
}

/// State of an object after execution
///
/// # BCS
///
/// The BCS serialized form for this type is defined by the following ABNF:
///
/// ```text
/// object-out  =  object-out-missing
///             =/ object-out-object-write
///             =/ object-out-package-write
///
///
/// object-out-missing        = %x00
/// object-out-object-write   = %x01 digest owner
/// object-out-package-write  = %x02 version digest
/// ```
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(
    feature = "schemars",
    derive(schemars::JsonSchema),
    schemars(tag = "state", rename_all = "snake_case")
)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[non_exhaustive]
pub enum ObjectOut {
    /// Same definition as in ObjectIn.
    Missing,
    /// Any written object, including all of mutated, created, unwrapped today.
    ObjectWrite { digest: Digest, owner: Owner },
    /// Packages writes need to be tracked separately with version because
    /// we don't use lamport version for package publish and upgrades.
    PackageWrite {
        #[cfg_attr(feature = "schemars", schemars(with = "crate::_schemars::U64"))]
        version: Version,
        digest: Digest,
    },
}

impl ObjectOut {
    crate::def_is!(Missing, ObjectWrite, PackageWrite);

    pub fn object_digest_opt(&self) -> Option<Digest> {
        if let Self::ObjectWrite { digest, .. } = self {
            Some(*digest)
        } else {
            None
        }
    }

    pub fn object_digest(&self) -> Digest {
        self.object_digest_opt().expect("object does not exist")
    }

    pub fn object_owner_opt(&self) -> Option<Owner> {
        if let Self::ObjectWrite { owner, .. } = self {
            Some(*owner)
        } else {
            None
        }
    }

    pub fn object_owner(&self) -> Owner {
        self.object_owner_opt().expect("object does not exist")
    }

    pub fn package_version_opt(&self) -> Option<Version> {
        if let Self::PackageWrite { version, .. } = self {
            Some(*version)
        } else {
            None
        }
    }

    pub fn package_version(&self) -> Version {
        self.package_version_opt().expect("object does not exist")
    }

    pub fn package_digest_opt(&self) -> Option<Digest> {
        if let Self::PackageWrite { digest, .. } = self {
            Some(*digest)
        } else {
            None
        }
    }

    pub fn package_digest(&self) -> Digest {
        self.package_digest_opt().expect("package does not exist")
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod serialization {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    #[derive(serde::Serialize)]
    struct ReadableTransactionEffectsV1Ref<'a> {
        #[serde(flatten)]
        status: &'a ExecutionStatus,
        #[serde(with = "crate::_serde::ReadableDisplay")]
        epoch: &'a EpochId,
        gas_used: &'a GasCostSummary,
        transaction_digest: &'a Digest,
        gas_object_index: &'a Option<u32>,
        events_digest: &'a Option<Digest>,
        dependencies: &'a Vec<Digest>,
        #[serde(with = "crate::_serde::ReadableDisplay")]
        lamport_version: &'a Version,
        changed_objects: &'a Vec<ChangedObject>,
        unchanged_shared_objects: &'a Vec<UnchangedSharedObject>,
        auxiliary_data_digest: &'a Option<Digest>,
    }

    #[derive(serde::Deserialize)]
    struct ReadableTransactionEffectsV1 {
        #[serde(flatten)]
        status: ExecutionStatus,
        #[serde(with = "crate::_serde::ReadableDisplay")]
        epoch: EpochId,
        gas_used: GasCostSummary,
        transaction_digest: Digest,
        gas_object_index: Option<u32>,
        events_digest: Option<Digest>,
        dependencies: Vec<Digest>,
        #[serde(with = "crate::_serde::ReadableDisplay")]
        lamport_version: Version,
        changed_objects: Vec<ChangedObject>,
        unchanged_shared_objects: Vec<UnchangedSharedObject>,
        auxiliary_data_digest: Option<Digest>,
    }

    #[derive(serde::Serialize)]
    struct BinaryTransactionEffectsV1Ref<'a> {
        status: &'a ExecutionStatus,
        epoch: &'a EpochId,
        gas_used: &'a GasCostSummary,
        transaction_digest: &'a Digest,
        gas_object_index: &'a Option<u32>,
        events_digest: &'a Option<Digest>,
        dependencies: &'a Vec<Digest>,
        lamport_version: &'a Version,
        changed_objects: &'a Vec<ChangedObject>,
        unchanged_shared_objects: &'a Vec<UnchangedSharedObject>,
        auxiliary_data_digest: &'a Option<Digest>,
    }

    #[derive(serde::Deserialize)]
    struct BinaryTransactionEffectsV1 {
        status: ExecutionStatus,
        epoch: EpochId,
        gas_used: GasCostSummary,
        transaction_digest: Digest,
        gas_object_index: Option<u32>,
        events_digest: Option<Digest>,
        dependencies: Vec<Digest>,
        lamport_version: Version,
        changed_objects: Vec<ChangedObject>,
        unchanged_shared_objects: Vec<UnchangedSharedObject>,
        auxiliary_data_digest: Option<Digest>,
    }

    impl Serialize for TransactionEffectsV1 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let Self {
                status,
                epoch,
                gas_used,
                transaction_digest,
                gas_object_index,
                events_digest,
                dependencies,
                lamport_version,
                changed_objects,
                unchanged_shared_objects,
                auxiliary_data_digest,
            } = self;
            if serializer.is_human_readable() {
                let readable = ReadableTransactionEffectsV1Ref {
                    status,
                    epoch,
                    gas_used,
                    transaction_digest,
                    gas_object_index,
                    events_digest,
                    dependencies,
                    lamport_version,
                    changed_objects,
                    unchanged_shared_objects,
                    auxiliary_data_digest,
                };
                readable.serialize(serializer)
            } else {
                let binary = BinaryTransactionEffectsV1Ref {
                    status,
                    epoch,
                    gas_used,
                    transaction_digest,
                    gas_object_index,
                    events_digest,
                    dependencies,
                    lamport_version,
                    changed_objects,
                    unchanged_shared_objects,
                    auxiliary_data_digest,
                };
                binary.serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for TransactionEffectsV1 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                let ReadableTransactionEffectsV1 {
                    status,
                    epoch,
                    gas_used,
                    transaction_digest,
                    gas_object_index,
                    events_digest,
                    dependencies,
                    lamport_version,
                    changed_objects,
                    unchanged_shared_objects,
                    auxiliary_data_digest,
                } = Deserialize::deserialize(deserializer)?;
                Ok(Self {
                    status,
                    epoch,
                    gas_used,
                    transaction_digest,
                    gas_object_index,
                    events_digest,
                    dependencies,
                    lamport_version,
                    changed_objects,
                    unchanged_shared_objects,
                    auxiliary_data_digest,
                })
            } else {
                let BinaryTransactionEffectsV1 {
                    status,
                    epoch,
                    gas_used,
                    transaction_digest,
                    gas_object_index,
                    events_digest,
                    dependencies,
                    lamport_version,
                    changed_objects,
                    unchanged_shared_objects,
                    auxiliary_data_digest,
                } = Deserialize::deserialize(deserializer)?;
                Ok(Self {
                    status,
                    epoch,
                    gas_used,
                    transaction_digest,
                    gas_object_index,
                    events_digest,
                    dependencies,
                    lamport_version,
                    changed_objects,
                    unchanged_shared_objects,
                    auxiliary_data_digest,
                })
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum ReadableUnchangedSharedKind {
        ReadOnlyRoot {
            #[serde(with = "crate::_serde::ReadableDisplay")]
            version: Version,
            digest: Digest,
        },
        MutateDeleted {
            #[serde(with = "crate::_serde::ReadableDisplay")]
            version: Version,
        },
        ReadDeleted {
            #[serde(with = "crate::_serde::ReadableDisplay")]
            version: Version,
        },
        Cancelled {
            #[serde(with = "crate::_serde::ReadableDisplay")]
            version: Version,
        },
        PerEpochConfig,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    enum BinaryUnchangedSharedKind {
        ReadOnlyRoot { version: Version, digest: Digest },
        MutateDeleted { version: Version },
        ReadDeleted { version: Version },
        Cancelled { version: Version },
        PerEpochConfig,
    }

    impl Serialize for UnchangedSharedKind {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                let readable = match self.clone() {
                    UnchangedSharedKind::ReadOnlyRoot { version, digest } => {
                        ReadableUnchangedSharedKind::ReadOnlyRoot { version, digest }
                    }
                    UnchangedSharedKind::MutateDeleted { version } => {
                        ReadableUnchangedSharedKind::MutateDeleted { version }
                    }
                    UnchangedSharedKind::ReadDeleted { version } => {
                        ReadableUnchangedSharedKind::ReadDeleted { version }
                    }
                    UnchangedSharedKind::Cancelled { version } => {
                        ReadableUnchangedSharedKind::Cancelled { version }
                    }
                    UnchangedSharedKind::PerEpochConfig => {
                        ReadableUnchangedSharedKind::PerEpochConfig
                    }
                };
                readable.serialize(serializer)
            } else {
                let binary = match self.clone() {
                    UnchangedSharedKind::ReadOnlyRoot { version, digest } => {
                        BinaryUnchangedSharedKind::ReadOnlyRoot { version, digest }
                    }
                    UnchangedSharedKind::MutateDeleted { version } => {
                        BinaryUnchangedSharedKind::MutateDeleted { version }
                    }
                    UnchangedSharedKind::ReadDeleted { version } => {
                        BinaryUnchangedSharedKind::ReadDeleted { version }
                    }
                    UnchangedSharedKind::Cancelled { version } => {
                        BinaryUnchangedSharedKind::Cancelled { version }
                    }
                    UnchangedSharedKind::PerEpochConfig => {
                        BinaryUnchangedSharedKind::PerEpochConfig
                    }
                };
                binary.serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for UnchangedSharedKind {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                ReadableUnchangedSharedKind::deserialize(deserializer).map(
                    |readable| match readable {
                        ReadableUnchangedSharedKind::ReadOnlyRoot { version, digest } => {
                            Self::ReadOnlyRoot { version, digest }
                        }
                        ReadableUnchangedSharedKind::MutateDeleted { version } => {
                            Self::MutateDeleted { version }
                        }
                        ReadableUnchangedSharedKind::ReadDeleted { version } => {
                            Self::ReadDeleted { version }
                        }
                        ReadableUnchangedSharedKind::Cancelled { version } => {
                            Self::Cancelled { version }
                        }
                        ReadableUnchangedSharedKind::PerEpochConfig => Self::PerEpochConfig,
                    },
                )
            } else {
                BinaryUnchangedSharedKind::deserialize(deserializer).map(|binary| match binary {
                    BinaryUnchangedSharedKind::ReadOnlyRoot { version, digest } => {
                        Self::ReadOnlyRoot { version, digest }
                    }
                    BinaryUnchangedSharedKind::MutateDeleted { version } => {
                        Self::MutateDeleted { version }
                    }
                    BinaryUnchangedSharedKind::ReadDeleted { version } => {
                        Self::ReadDeleted { version }
                    }
                    BinaryUnchangedSharedKind::Cancelled { version } => Self::Cancelled { version },
                    BinaryUnchangedSharedKind::PerEpochConfig => Self::PerEpochConfig,
                })
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "state", rename_all = "snake_case")]
    enum ReadableObjectIn {
        Missing,
        Data {
            #[serde(with = "crate::_serde::ReadableDisplay")]
            version: Version,
            digest: Digest,
            owner: Owner,
        },
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    enum BinaryObjectIn {
        Missing,
        Data {
            version: Version,
            digest: Digest,
            owner: Owner,
        },
    }

    impl Serialize for ObjectIn {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                let readable = match self.clone() {
                    ObjectIn::Missing => ReadableObjectIn::Missing,
                    ObjectIn::Data {
                        version,
                        digest,
                        owner,
                    } => ReadableObjectIn::Data {
                        version,
                        digest,
                        owner,
                    },
                };
                readable.serialize(serializer)
            } else {
                let binary = match self.clone() {
                    ObjectIn::Missing => BinaryObjectIn::Missing,
                    ObjectIn::Data {
                        version,
                        digest,
                        owner,
                    } => BinaryObjectIn::Data {
                        version,
                        digest,
                        owner,
                    },
                };
                binary.serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for ObjectIn {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                ReadableObjectIn::deserialize(deserializer).map(|readable| match readable {
                    ReadableObjectIn::Missing => Self::Missing,
                    ReadableObjectIn::Data {
                        version,
                        digest,
                        owner,
                    } => Self::Data {
                        version,
                        digest,
                        owner,
                    },
                })
            } else {
                BinaryObjectIn::deserialize(deserializer).map(|binary| match binary {
                    BinaryObjectIn::Missing => Self::Missing,
                    BinaryObjectIn::Data {
                        version,
                        digest,
                        owner,
                    } => Self::Data {
                        version,
                        digest,
                        owner,
                    },
                })
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "state", rename_all = "snake_case")]
    enum ReadableObjectOut {
        Missing,
        ObjectWrite {
            digest: Digest,
            owner: Owner,
        },
        PackageWrite {
            #[serde(with = "crate::_serde::ReadableDisplay")]
            version: Version,
            digest: Digest,
        },
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    enum BinaryObjectOut {
        Missing,
        ObjectWrite {
            digest: Digest,
            owner: Owner,
        },
        PackageWrite {
            #[serde(with = "crate::_serde::ReadableDisplay")]
            version: Version,
            digest: Digest,
        },
    }

    impl Serialize for ObjectOut {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                let readable = match self.clone() {
                    ObjectOut::Missing => ReadableObjectOut::Missing,
                    ObjectOut::ObjectWrite { digest, owner } => {
                        ReadableObjectOut::ObjectWrite { digest, owner }
                    }
                    ObjectOut::PackageWrite { version, digest } => {
                        ReadableObjectOut::PackageWrite { version, digest }
                    }
                };
                readable.serialize(serializer)
            } else {
                let binary = match self.clone() {
                    ObjectOut::Missing => BinaryObjectOut::Missing,
                    ObjectOut::ObjectWrite { digest, owner } => {
                        BinaryObjectOut::ObjectWrite { digest, owner }
                    }
                    ObjectOut::PackageWrite { version, digest } => {
                        BinaryObjectOut::PackageWrite { version, digest }
                    }
                };
                binary.serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for ObjectOut {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                ReadableObjectOut::deserialize(deserializer).map(|readable| match readable {
                    ReadableObjectOut::Missing => Self::Missing,
                    ReadableObjectOut::ObjectWrite { digest, owner } => {
                        Self::ObjectWrite { digest, owner }
                    }
                    ReadableObjectOut::PackageWrite { version, digest } => {
                        Self::PackageWrite { version, digest }
                    }
                })
            } else {
                BinaryObjectOut::deserialize(deserializer).map(|binary| match binary {
                    BinaryObjectOut::Missing => Self::Missing,
                    BinaryObjectOut::ObjectWrite { digest, owner } => {
                        Self::ObjectWrite { digest, owner }
                    }
                    BinaryObjectOut::PackageWrite { version, digest } => {
                        Self::PackageWrite { version, digest }
                    }
                })
            }
        }
    }
}
