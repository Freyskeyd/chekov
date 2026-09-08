//! A small, domain-neutral state model for deterministic execution.
//!
//! The core deliberately models only opaque bytes, revisions, observations, and
//! mutations. Domain-specific adapters are responsible for encoding keys and
//! values and for assigning meaning to revisions.

use std::collections::BTreeMap;
use std::sync::Arc;

/// An opaque, cheaply shareable key into state.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StateKey(Arc<[u8]>);

impl StateKey {
    /// Creates a key by copying the supplied bytes.
    pub fn new(bytes: impl AsRef<[u8]>) -> Self {
        Self(Arc::from(bytes.as_ref()))
    }

    /// Creates a key from an existing shared byte buffer without copying it.
    pub fn from_arc(bytes: Arc<[u8]>) -> Self {
        Self(bytes)
    }

    /// Returns the opaque key bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the shared byte buffer.
    pub fn into_arc(self) -> Arc<[u8]> {
        self.0
    }
}

impl AsRef<[u8]> for StateKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Arc<[u8]>> for StateKey {
    fn from(bytes: Arc<[u8]>) -> Self {
        Self::from_arc(bytes)
    }
}

impl From<Vec<u8>> for StateKey {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl From<&[u8]> for StateKey {
    fn from(bytes: &[u8]) -> Self {
        Self::new(bytes)
    }
}

impl From<&str> for StateKey {
    fn from(bytes: &str) -> Self {
        Self::new(bytes.as_bytes())
    }
}

/// Opaque, immutable, and cheaply shareable state data.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StateValue(Arc<[u8]>);

impl StateValue {
    /// Creates a value by copying the supplied bytes.
    pub fn new(bytes: impl AsRef<[u8]>) -> Self {
        Self(Arc::from(bytes.as_ref()))
    }

    /// Creates a value from an existing shared byte buffer without copying it.
    pub fn from_arc(bytes: Arc<[u8]>) -> Self {
        Self(bytes)
    }

    /// Returns the opaque value bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns the shared byte buffer.
    pub fn into_arc(self) -> Arc<[u8]> {
        self.0
    }
}

impl AsRef<[u8]> for StateValue {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Arc<[u8]>> for StateValue {
    fn from(bytes: Arc<[u8]>) -> Self {
        Self::from_arc(bytes)
    }
}

impl From<Vec<u8>> for StateValue {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl From<&[u8]> for StateValue {
    fn from(bytes: &[u8]) -> Self {
        Self::new(bytes)
    }
}

impl From<&str> for StateValue {
    fn from(bytes: &str) -> Self {
        Self::new(bytes.as_bytes())
    }
}

/// A concurrency token associated with a state cell.
///
/// A revision is not a domain version. Domain adapters may choose to map their
/// own versions to revisions, but the core does not assign that meaning.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

impl Revision {
    /// The initial revision for a key that has never been written.
    pub const ZERO: Self = Self(0);

    /// Creates a revision from its raw token.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw revision token.
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Returns the next revision, or `None` if the token is exhausted.
    pub const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl From<u64> for Revision {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<Revision> for u64 {
    fn from(revision: Revision) -> Self {
        revision.value()
    }
}

/// The value and concurrency token observed for one key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateCell {
    /// The concurrency token observed with this cell.
    pub revision: Revision,
    /// The opaque value, or `None` when the key is absent.
    pub value: Option<StateValue>,
}

impl StateCell {
    /// Creates a cell. An absent cell still has a meaningful revision.
    pub const fn new(revision: Revision, value: Option<StateValue>) -> Self {
        Self { revision, value }
    }

    /// Creates an absent cell at `revision`.
    pub const fn absent(revision: Revision) -> Self {
        Self::new(revision, None)
    }

    /// Creates a present cell at `revision`.
    pub const fn present(revision: Revision, value: StateValue) -> Self {
        Self::new(revision, Some(value))
    }

    /// Returns whether this cell contains a value.
    pub const fn is_present(&self) -> bool {
        self.value.is_some()
    }
}

/// A synchronous read-only view of state.
pub trait StateView {
    /// The error returned when a read cannot be completed.
    type Error;

    /// Reads one key and returns its value and revision.
    fn read(&self, key: &StateKey) -> Result<StateCell, Self::Error>;
}

/// A revision observed while executing against a state view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    /// The key whose revision was observed.
    pub key: StateKey,
    /// The revision returned by the state view.
    pub revision: Revision,
}

impl Observation {
    /// Creates an observation for a key and its revision.
    pub fn new(key: StateKey, revision: Revision) -> Self {
        Self { key, revision }
    }
}

/// Deterministic set of key revisions observed during execution.
pub type ReadSet = BTreeMap<StateKey, Revision>;

/// A single compare-and-swap state change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mutation {
    /// The key to change.
    pub key: StateKey,
    /// The revision the mutation was produced against.
    pub expected_revision: Revision,
    /// The value expected at `expected_revision`.
    pub before: Option<StateValue>,
    /// The value to publish, or `None` to delete the cell.
    pub after: Option<StateValue>,
}

impl Mutation {
    /// Creates an explicit state change from `before` to `after`.
    pub fn new(
        key: StateKey,
        expected_revision: Revision,
        before: Option<StateValue>,
        after: Option<StateValue>,
    ) -> Self {
        Self {
            key,
            expected_revision,
            before,
            after,
        }
    }

    /// Creates a mutation for a previously absent key.
    pub fn create(key: StateKey, expected_revision: Revision, value: StateValue) -> Self {
        Self::new(key, expected_revision, None, Some(value))
    }

    /// Creates a mutation that replaces an existing value.
    pub fn update(
        key: StateKey,
        expected_revision: Revision,
        before: StateValue,
        after: StateValue,
    ) -> Self {
        Self::new(key, expected_revision, Some(before), Some(after))
    }

    /// Creates a mutation that removes an existing value.
    pub fn delete(key: StateKey, expected_revision: Revision, before: StateValue) -> Self {
        Self::new(key, expected_revision, Some(before), None)
    }
}

/// The domain-neutral result of deterministic state execution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransitionBatch {
    /// Revisions observed while producing this batch.
    pub reads: ReadSet,
    /// Canonical state changes, in execution order.
    pub mutations: Vec<Mutation>,
}

impl TransitionBatch {
    /// Creates a transition batch from its read set and mutations.
    pub fn new(reads: ReadSet, mutations: Vec<Mutation>) -> Self {
        Self { reads, mutations }
    }

    /// Creates an empty transition batch.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Records the first revision observed for a key.
    pub fn record_observation(&mut self, observation: Observation) {
        self.reads
            .entry(observation.key)
            .or_insert(observation.revision);
    }

    /// Adds a mutation to the end of the batch.
    pub fn push_mutation(&mut self, mutation: Mutation) {
        self.mutations.push(mutation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    #[derive(Debug, Eq, PartialEq)]
    enum MemoryError {
        RevisionMismatch {
            key: StateKey,
            expected: Revision,
            actual: Revision,
        },
        BeforeMismatch {
            key: StateKey,
        },
        RevisionExhausted,
    }

    #[derive(Default)]
    struct MemoryState {
        cells: BTreeMap<StateKey, StateCell>,
    }

    impl StateView for MemoryState {
        type Error = Infallible;

        fn read(&self, key: &StateKey) -> Result<StateCell, Self::Error> {
            Ok(self
                .cells
                .get(key)
                .cloned()
                .unwrap_or_else(|| StateCell::absent(Revision::ZERO)))
        }
    }

    impl MemoryState {
        fn apply(&mut self, mutation: Mutation) -> Result<Revision, MemoryError> {
            let current = self.read(&mutation.key).expect("memory reads cannot fail");

            if current.revision != mutation.expected_revision {
                return Err(MemoryError::RevisionMismatch {
                    key: mutation.key,
                    expected: mutation.expected_revision,
                    actual: current.revision,
                });
            }

            if current.value != mutation.before {
                return Err(MemoryError::BeforeMismatch { key: mutation.key });
            }

            let revision = current
                .revision
                .checked_next()
                .ok_or(MemoryError::RevisionExhausted)?;
            self.cells
                .insert(mutation.key, StateCell::new(revision, mutation.after));
            Ok(revision)
        }
    }

    fn key(bytes: &'static [u8]) -> StateKey {
        StateKey::from(bytes)
    }

    fn value(bytes: &'static [u8]) -> StateValue {
        StateValue::from(bytes)
    }

    #[test]
    fn memory_state_supports_creation_update_and_deletion() {
        let mut state = MemoryState::default();
        let key = key(b"key");
        let first = value(b"one");
        let second = value(b"two");

        assert_eq!(state.read(&key).unwrap(), StateCell::absent(Revision::ZERO));

        let revision = state
            .apply(Mutation::create(key.clone(), Revision::ZERO, first.clone()))
            .unwrap();
        assert_eq!(revision, Revision::new(1));
        assert_eq!(
            state.read(&key).unwrap(),
            StateCell::present(Revision::new(1), first.clone())
        );

        let revision = state
            .apply(Mutation::update(
                key.clone(),
                revision,
                first.clone(),
                second.clone(),
            ))
            .unwrap();
        assert_eq!(revision, Revision::new(2));

        let revision = state
            .apply(Mutation::delete(key.clone(), revision, second))
            .unwrap();
        assert_eq!(revision, Revision::new(3));
        assert_eq!(state.read(&key).unwrap(), StateCell::absent(revision));
    }

    #[test]
    fn stale_observations_are_rejected_even_when_the_old_cell_was_absent() {
        let mut state = MemoryState::default();
        let key = key(b"key");
        let observation = state.read(&key).unwrap();

        state
            .apply(Mutation::create(
                key.clone(),
                observation.revision,
                value(b"created"),
            ))
            .unwrap();

        let stale = state.apply(Mutation::new(
            key.clone(),
            observation.revision,
            observation.value,
            Some(value(b"stale write")),
        ));
        assert_eq!(
            stale,
            Err(MemoryError::RevisionMismatch {
                key,
                expected: Revision::ZERO,
                actual: Revision::new(1),
            })
        );
    }

    #[test]
    fn transition_batch_records_reads_and_explicit_mutations() {
        let key = key(b"key");
        let mut transition = TransitionBatch::empty();
        transition.record_observation(Observation::new(key.clone(), Revision::new(4)));
        transition.record_observation(Observation::new(key.clone(), Revision::new(9)));
        transition.push_mutation(Mutation::new(
            key.clone(),
            Revision::new(4),
            None,
            Some(value(b"value")),
        ));

        assert_eq!(transition.reads.get(&key), Some(&Revision::new(4)));
        assert_eq!(transition.mutations.len(), 1);
        assert_eq!(transition.mutations[0].key, key);
        assert_eq!(transition.mutations[0].before, None);
        assert_eq!(transition.mutations[0].after, Some(value(b"value")));
    }

    #[test]
    fn memory_state_rejects_revision_overflow() {
        let mut state = MemoryState::default();
        let key = key(b"key");
        let revision = Revision::new(u64::MAX);
        state
            .cells
            .insert(key.clone(), StateCell::present(revision, value(b"value")));

        assert_eq!(revision.checked_next(), None);
        assert_eq!(
            state.apply(Mutation::update(
                key.clone(),
                revision,
                value(b"value"),
                value(b"next"),
            )),
            Err(MemoryError::RevisionExhausted)
        );
        assert_eq!(state.read(&key).unwrap().revision, revision);
    }
}
