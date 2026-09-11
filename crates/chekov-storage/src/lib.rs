//! Storage contracts for applying deterministic state transitions.

use chekov_core::{Revision, StateCell, StateKey, StateView, TransitionBatch};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// A failed compare-and-swap validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageError {
    /// The observed revision is no longer the revision required by a read or mutation.
    Conflict {
        key: StateKey,
        expected: Revision,
        actual: Revision,
    },
    /// A mutation's value does not match the value at its expected revision.
    ValueMismatch {
        key: StateKey,
        expected: Option<chekov_core::StateValue>,
        actual: Option<chekov_core::StateValue>,
    },
    /// A key cannot advance beyond the maximum representable revision.
    RevisionExhausted { key: StateKey, revision: Revision },
    /// The storage lock was poisoned by a panic while another operation held it.
    LockPoisoned,
}

/// One mutation successfully published by a commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedMutation {
    pub key: StateKey,
    pub cell: StateCell,
}

/// The deterministic result of applying a transition batch.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppliedBatch {
    pub mutations: Vec<AppliedMutation>,
}

/// A storage implementation that validates and commits a whole transition batch.
pub trait VersionedStorage: StateView {
    /// Applies all mutations, or leaves canonical state untouched on failure.
    fn apply(&self, batch: &TransitionBatch) -> Result<AppliedBatch, Self::Error>;
}

/// An in-memory, atomically updated state store.
///
/// The lock covers validation and commit together. Clones share canonical state,
/// so separate execution handles still participate in the same compare-and-swap
/// contract.
#[derive(Clone, Debug, Default)]
pub struct MemoryStorage {
    cells: Arc<RwLock<BTreeMap<StateKey, StateCell>>>,
}

impl MemoryStorage {
    /// Applies a transition batch atomically.
    pub fn apply(&self, batch: &TransitionBatch) -> Result<AppliedBatch, StorageError> {
        <Self as VersionedStorage>::apply(self, batch)
    }

    /// Returns the current state for a key.
    pub fn read(&self, key: &StateKey) -> Result<StateCell, StorageError> {
        <Self as StateView>::read(self, key)
    }
}

impl StateView for MemoryStorage {
    type Error = StorageError;

    fn read(&self, key: &StateKey) -> Result<StateCell, Self::Error> {
        let cells = self.cells.read().map_err(|_| StorageError::LockPoisoned)?;

        Ok(cells
            .get(key)
            .cloned()
            .unwrap_or_else(|| StateCell::absent(Revision::ZERO)))
    }
}

impl VersionedStorage for MemoryStorage {
    fn apply(&self, batch: &TransitionBatch) -> Result<AppliedBatch, Self::Error> {
        let mut cells = self.cells.write().map_err(|_| StorageError::LockPoisoned)?;

        // Validate against a batch-local overlay first. Nothing in the
        // canonical map is changed until every read and mutation succeeds.
        let mut overlay = BTreeMap::new();
        for (key, expected_revision) in &batch.reads {
            let current = cell_at(&cells, &overlay, key);
            ensure_revision(key, *expected_revision, current.revision)?;
        }

        let mut applied = Vec::with_capacity(batch.mutations.len());
        for mutation in &batch.mutations {
            let current = cell_at(&cells, &overlay, &mutation.key);
            ensure_revision(&mutation.key, mutation.expected_revision, current.revision)?;

            if current.value != mutation.before {
                return Err(StorageError::ValueMismatch {
                    key: mutation.key.clone(),
                    expected: mutation.before.clone(),
                    actual: current.value.clone(),
                });
            }

            let revision =
                current
                    .revision
                    .checked_next()
                    .ok_or_else(|| StorageError::RevisionExhausted {
                        key: mutation.key.clone(),
                        revision: current.revision,
                    })?;
            let cell = StateCell::new(revision, mutation.after.clone());
            overlay.insert(mutation.key.clone(), cell.clone());
            applied.push(AppliedMutation {
                key: mutation.key.clone(),
                cell,
            });
        }

        cells.extend(overlay);
        Ok(AppliedBatch { mutations: applied })
    }
}

fn cell_at(
    cells: &BTreeMap<StateKey, StateCell>,
    overlay: &BTreeMap<StateKey, StateCell>,
    key: &StateKey,
) -> StateCell {
    overlay
        .get(key)
        .or_else(|| cells.get(key))
        .cloned()
        .unwrap_or_else(|| StateCell::absent(Revision::ZERO))
}

fn ensure_revision(
    key: &StateKey,
    expected: Revision,
    actual: Revision,
) -> Result<(), StorageError> {
    if expected == actual {
        Ok(())
    } else {
        Err(StorageError::Conflict {
            key: key.clone(),
            expected,
            actual,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chekov_core::{Mutation, StateValue};
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn key(value: &'static str) -> StateKey {
        StateKey::from(value)
    }

    fn value(value: &'static str) -> StateValue {
        StateValue::from(value)
    }

    fn create(key: StateKey, value: StateValue) -> TransitionBatch {
        TransitionBatch::new(
            [(key.clone(), Revision::ZERO)].into_iter().collect(),
            vec![Mutation::create(key, Revision::ZERO, value)],
        )
    }

    #[test]
    fn creates_updates_and_deletes_without_losing_tombstone_revisions() {
        let storage = MemoryStorage::default();
        let key = key("account");
        let first = value("one");
        let second = value("two");

        assert_eq!(
            storage.read(&key).unwrap(),
            StateCell::absent(Revision::ZERO)
        );
        storage.apply(&create(key.clone(), first.clone())).unwrap();
        storage
            .apply(&TransitionBatch::new(
                [(key.clone(), Revision::new(1))].into_iter().collect(),
                vec![Mutation::update(
                    key.clone(),
                    Revision::new(1),
                    first,
                    second.clone(),
                )],
            ))
            .unwrap();
        storage
            .apply(&TransitionBatch::new(
                [(key.clone(), Revision::new(2))].into_iter().collect(),
                vec![Mutation::delete(key.clone(), Revision::new(2), second)],
            ))
            .unwrap();

        assert_eq!(
            storage.read(&key).unwrap(),
            StateCell::absent(Revision::new(3))
        );
    }

    #[test]
    fn stale_multi_key_batch_does_not_partially_commit() {
        let storage = MemoryStorage::default();
        let first_key = key("first");
        let second_key = key("second");
        storage
            .apply(&create(first_key.clone(), value("original-first")))
            .unwrap();
        storage
            .apply(&create(second_key.clone(), value("original-second")))
            .unwrap();

        let stale = TransitionBatch::new(
            [
                (first_key.clone(), Revision::new(1)),
                (second_key.clone(), Revision::ZERO),
            ]
            .into_iter()
            .collect(),
            vec![
                Mutation::update(
                    first_key.clone(),
                    Revision::new(1),
                    value("original-first"),
                    value("changed-first"),
                ),
                Mutation::update(
                    second_key.clone(),
                    Revision::ZERO,
                    value("original-second"),
                    value("changed-second"),
                ),
            ],
        );

        assert_eq!(
            storage.apply(&stale),
            Err(StorageError::Conflict {
                key: second_key.clone(),
                expected: Revision::ZERO,
                actual: Revision::new(1),
            })
        );
        assert_eq!(
            storage.read(&first_key).unwrap(),
            StateCell::present(Revision::new(1), value("original-first"))
        );
        assert_eq!(
            storage.read(&second_key).unwrap(),
            StateCell::present(Revision::new(1), value("original-second"))
        );
    }

    #[test]
    fn failed_mutation_validation_does_not_commit_earlier_mutations() {
        let storage = MemoryStorage::default();
        let first_key = key("first");
        let second_key = key("second");
        storage
            .apply(&create(first_key.clone(), value("original-first")))
            .unwrap();
        storage
            .apply(&create(second_key.clone(), value("original-second")))
            .unwrap();

        let batch = TransitionBatch::new(
            [(first_key.clone(), Revision::new(1))]
                .into_iter()
                .collect(),
            vec![
                Mutation::update(
                    first_key.clone(),
                    Revision::new(1),
                    value("original-first"),
                    value("changed-first"),
                ),
                Mutation::update(
                    second_key.clone(),
                    Revision::new(1),
                    value("stale-second-value"),
                    value("changed-second"),
                ),
            ],
        );

        assert!(matches!(
            storage.apply(&batch),
            Err(StorageError::ValueMismatch { key, .. }) if key == second_key
        ));
        assert_eq!(
            storage.read(&first_key).unwrap(),
            StateCell::present(Revision::new(1), value("original-first"))
        );
    }

    #[test]
    fn concurrent_stale_batches_have_one_winner() {
        let storage = Arc::new(MemoryStorage::default());
        let key = key("counter");
        storage.apply(&create(key.clone(), value("zero"))).unwrap();

        let first = Arc::clone(&storage);
        let second = Arc::clone(&storage);
        let barrier = Arc::new(Barrier::new(3));
        let first_barrier = Arc::clone(&barrier);
        let second_barrier = Arc::clone(&barrier);
        let first_batch = TransitionBatch::new(
            [(key.clone(), Revision::new(1))].into_iter().collect(),
            vec![Mutation::update(
                key.clone(),
                Revision::new(1),
                value("zero"),
                value("first"),
            )],
        );
        let second_batch = TransitionBatch::new(
            [(key.clone(), Revision::new(1))].into_iter().collect(),
            vec![Mutation::update(
                key.clone(),
                Revision::new(1),
                value("zero"),
                value("second"),
            )],
        );

        let first_handle = thread::spawn(move || {
            first_barrier.wait();
            first.apply(&first_batch)
        });
        let second_handle = thread::spawn(move || {
            second_barrier.wait();
            second.apply(&second_batch)
        });
        barrier.wait();
        let first_result = first_handle.join().unwrap();
        let second_result = second_handle.join().unwrap();
        assert!(matches!(
            (first_result, second_result),
            (Ok(_), Err(StorageError::Conflict { .. }))
                | (Err(StorageError::Conflict { .. }), Ok(_))
        ));
        assert_eq!(storage.read(&key).unwrap().revision, Revision::new(2));
    }

    #[test]
    fn revision_overflow_does_not_commit_any_mutation() {
        let storage = MemoryStorage::default();
        let exhausted = key("exhausted");
        let ordinary = key("ordinary");
        storage.cells.write().unwrap().insert(
            exhausted.clone(),
            StateCell::present(Revision::new(u64::MAX), value("old")),
        );

        let batch = TransitionBatch::new(
            [
                (exhausted.clone(), Revision::new(u64::MAX)),
                (ordinary.clone(), Revision::ZERO),
            ]
            .into_iter()
            .collect(),
            vec![
                Mutation::update(
                    exhausted.clone(),
                    Revision::new(u64::MAX),
                    value("old"),
                    value("new"),
                ),
                Mutation::create(ordinary.clone(), Revision::ZERO, value("created")),
            ],
        );

        assert!(matches!(
            storage.apply(&batch),
            Err(StorageError::RevisionExhausted { key, .. }) if key == exhausted
        ));
        assert_eq!(
            storage.read(&exhausted).unwrap(),
            StateCell::present(Revision::new(u64::MAX), value("old"))
        );
        assert_eq!(
            storage.read(&ordinary).unwrap(),
            StateCell::absent(Revision::ZERO)
        );
    }
}
