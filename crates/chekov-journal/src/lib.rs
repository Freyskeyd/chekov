//! Transactional state overlays with nested commit and revert semantics.
//!
//! A [`Journal`] keeps tentative writes in checkpoint-local maps while sharing
//! immutable values with the base [`StateView`]. Reads from the base view are
//! retained as observations for the lifetime of the transaction, including
//! reads made by a reverted checkpoint.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub use chekov_core::{
    Mutation, Observation, ReadSet, Revision, StateCell, StateKey, StateValue, StateView,
    TransitionBatch,
};

static NEXT_JOURNAL_ID: AtomicU64 = AtomicU64::new(1);

/// A non-cloneable token identifying one open checkpoint.
///
/// A token is marked closed by [`Journal::commit`] or [`Journal::revert`].
/// Tokens must be closed in last-in, first-out order; rejected operations leave
/// the token open so the caller can retry after closing a child checkpoint.
#[derive(Debug, Eq, PartialEq)]
pub struct Checkpoint {
    journal_id: u64,
    id: u64,
    closed: bool,
}

#[derive(Debug)]
struct PendingWrite {
    expected_revision: Revision,
    before: Option<StateValue>,
    after: Option<StateValue>,
}

#[derive(Debug)]
struct Frame {
    checkpoint_id: Option<u64>,
    writes: BTreeMap<StateKey, PendingWrite>,
    order: Vec<StateKey>,
}

impl Frame {
    fn root() -> Self {
        Self {
            checkpoint_id: None,
            writes: BTreeMap::new(),
            order: Vec::new(),
        }
    }

    fn checkpoint(id: u64) -> Self {
        Self {
            checkpoint_id: Some(id),
            writes: BTreeMap::new(),
            order: Vec::new(),
        }
    }
}

fn make_mutation(
    key: StateKey,
    expected_revision: Revision,
    before: Option<StateValue>,
    after: Option<StateValue>,
) -> Option<Mutation> {
    (before.as_ref() != after.as_ref())
        .then(|| Mutation::new(key, expected_revision, before, after))
}

/// Errors caused by closing a checkpoint out of order or on the wrong journal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointError {
    /// The token belongs to another journal.
    ForeignCheckpoint,
    /// The token is not the currently active (top) checkpoint.
    NotTopCheckpoint,
    /// The checkpoint was already committed or reverted.
    AlreadyClosed,
}

/// A transactional overlay over a deterministic canonical state view.
pub struct Journal<S> {
    base: S,
    journal_id: u64,
    next_checkpoint_id: u64,
    frames: Vec<Frame>,
    observations: ReadSet,
    canonical_reads: BTreeMap<StateKey, StateCell>,
}

impl<S> Journal<S> {
    /// Creates an empty transaction over `base`.
    pub fn new(base: S) -> Self {
        Self {
            base,
            journal_id: NEXT_JOURNAL_ID.fetch_add(1, Ordering::Relaxed),
            next_checkpoint_id: 0,
            frames: vec![Frame::root()],
            observations: BTreeMap::new(),
            canonical_reads: BTreeMap::new(),
        }
    }

    /// Returns the canonical read observations collected so far.
    pub fn observations(&self) -> &ReadSet {
        &self.observations
    }

    /// Opens a nested checkpoint.
    pub fn checkpoint(&mut self) -> Checkpoint {
        let id = self.next_checkpoint_id;
        self.next_checkpoint_id = self
            .next_checkpoint_id
            .checked_add(1)
            .expect("journal checkpoint limit exceeded");
        self.frames.push(Frame::checkpoint(id));
        Checkpoint {
            journal_id: self.journal_id,
            id,
            closed: false,
        }
    }

    /// Commits the top checkpoint into its parent overlay.
    pub fn commit(&mut self, checkpoint: &mut Checkpoint) -> Result<(), CheckpointError> {
        self.close(checkpoint, true)
    }

    /// Reverts the top checkpoint and discards only its tentative writes.
    pub fn revert(&mut self, checkpoint: &mut Checkpoint) -> Result<(), CheckpointError> {
        self.close(checkpoint, false)
    }

    /// Reads through the current overlay and records canonical reads.
    pub fn read(&mut self, key: &StateKey) -> Result<StateCell, S::Error>
    where
        S: StateView,
    {
        if let Some(write) = self.pending_write(key) {
            return Ok(StateCell::new(write.expected_revision, write.after.clone()));
        }

        if let Some(cell) = self.canonical_reads.get(key) {
            return Ok(cell.clone());
        }

        let cell = self.base.read(key)?;
        self.observations
            .entry(key.clone())
            .or_insert(cell.revision);
        self.canonical_reads.insert(key.clone(), cell.clone());
        Ok(cell)
    }

    /// Records a tentative value for `key`, or a deletion when `value` is `None`.
    pub fn write<K>(&mut self, key: K, value: Option<StateValue>) -> Result<(), S::Error>
    where
        S: StateView,
        K: Into<StateKey>,
    {
        let key = key.into();
        let current = self.read(&key)?;
        let frame = self
            .frames
            .last_mut()
            .expect("journal always has a root frame");

        match frame.writes.get_mut(&key) {
            Some(write) => write.after = value,
            None => {
                frame.order.push(key.clone());
                frame.writes.insert(
                    key,
                    PendingWrite {
                        expected_revision: current.revision,
                        before: current.value,
                        after: value,
                    },
                );
            }
        }

        Ok(())
    }

    /// Records a tentative value for `key`.
    pub fn set<K>(&mut self, key: K, value: StateValue) -> Result<(), S::Error>
    where
        S: StateView,
        K: Into<StateKey>,
    {
        self.write(key, Some(value))
    }

    /// Records a tentative deletion for `key`.
    pub fn delete<K>(&mut self, key: K) -> Result<(), S::Error>
    where
        S: StateView,
        K: Into<StateKey>,
    {
        self.write(key, None)
    }

    /// Builds the final batch without consuming the journal.
    ///
    /// An open checkpoint is rejected because its outcome has not yet been
    /// decided. Mutations retain the order of their first write, including
    /// writes merged from committed child checkpoints.
    pub fn transition(&self) -> Result<TransitionBatch, CheckpointError> {
        if self.frames.len() != 1 {
            return Err(CheckpointError::NotTopCheckpoint);
        }

        let root = &self.frames[0];
        let mutations = root
            .order
            .iter()
            .filter_map(|key| {
                root.writes.get(key).and_then(|write| {
                    make_mutation(
                        key.clone(),
                        write.expected_revision,
                        write.before.clone(),
                        write.after.clone(),
                    )
                })
            })
            .collect();

        Ok(TransitionBatch::new(self.observations.clone(), mutations))
    }

    /// Builds the final batch and consumes the journal.
    pub fn into_transition(self) -> Result<TransitionBatch, CheckpointError> {
        if self.frames.len() != 1 {
            return Err(CheckpointError::NotTopCheckpoint);
        }

        let Journal {
            observations,
            frames,
            ..
        } = self;
        let Frame { writes, order, .. } = frames
            .into_iter()
            .next()
            .expect("journal always has a root frame");
        let mut writes = writes;
        let mutations = order
            .into_iter()
            .filter_map(|key| {
                let PendingWrite {
                    expected_revision,
                    before,
                    after,
                } = writes
                    .remove(&key)
                    .expect("write order matches the write map");
                make_mutation(key, expected_revision, before, after)
            })
            .collect();

        Ok(TransitionBatch::new(observations, mutations))
    }

    fn pending_write(&self, key: &StateKey) -> Option<&PendingWrite> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.writes.get(key))
    }

    fn close(&mut self, checkpoint: &mut Checkpoint, commit: bool) -> Result<(), CheckpointError> {
        if checkpoint.journal_id != self.journal_id {
            return Err(CheckpointError::ForeignCheckpoint);
        }

        if checkpoint.closed {
            return Err(CheckpointError::AlreadyClosed);
        }

        let active = self.frames.last().and_then(|frame| frame.checkpoint_id);
        if active != Some(checkpoint.id) {
            return Err(CheckpointError::NotTopCheckpoint);
        }

        checkpoint.closed = true;
        let mut child = self.frames.pop().expect("active checkpoint has a frame");
        if commit {
            let parent = self.frames.last_mut().expect("checkpoint has a parent");
            for key in child.order {
                let child_write = child
                    .writes
                    .remove(&key)
                    .expect("checkpoint write order matches its write map");
                match parent.writes.get_mut(&key) {
                    Some(parent_write) => parent_write.after = child_write.after,
                    None => {
                        parent.order.push(key.clone());
                        parent.writes.insert(key, child_write);
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    #[derive(Default)]
    struct MemoryState {
        cells: BTreeMap<StateKey, StateCell>,
    }

    impl MemoryState {
        fn with(entries: impl IntoIterator<Item = (&'static str, u64, &'static str)>) -> Self {
            Self {
                cells: entries
                    .into_iter()
                    .map(|(key, revision, value)| {
                        (
                            StateKey::from(key),
                            StateCell::present(Revision::new(revision), StateValue::from(value)),
                        )
                    })
                    .collect(),
            }
        }
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

    fn value(value: &'static str) -> StateValue {
        StateValue::from(value)
    }

    #[test]
    fn nested_commit_merges_and_revert_discards_writes() {
        let base = MemoryState::with([("a", 7, "canonical")]);
        let mut journal = Journal::new(base);

        let mut outer = journal.checkpoint();
        journal.set("a", value("outer")).unwrap();
        let mut inner = journal.checkpoint();
        journal.set("a", value("inner")).unwrap();
        journal.set("b", value("discarded")).unwrap();
        assert_eq!(
            journal.read(&StateKey::from("a")).unwrap().value,
            Some(value("inner"))
        );
        journal.revert(&mut inner).unwrap();

        assert_eq!(
            journal.read(&StateKey::from("a")).unwrap().value,
            Some(value("outer"))
        );
        assert_eq!(journal.read(&StateKey::from("b")).unwrap().value, None);
        journal.commit(&mut outer).unwrap();

        let batch = journal.transition().unwrap();
        assert_eq!(batch.mutations.len(), 1);
        assert_eq!(batch.mutations[0].before, Some(value("canonical")));
        assert_eq!(batch.mutations[0].after, Some(value("outer")));
    }

    #[test]
    fn reads_from_reverted_scopes_are_retained() {
        let mut journal = Journal::new(MemoryState::with([("read", 12, "value")]));
        let mut checkpoint = journal.checkpoint();

        assert_eq!(
            journal.read(&StateKey::from("read")).unwrap().revision,
            Revision::new(12)
        );
        journal.revert(&mut checkpoint).unwrap();

        assert_eq!(
            journal.observations().get(&StateKey::from("read")),
            Some(&Revision::new(12))
        );
        assert!(journal.transition().unwrap().mutations.is_empty());
    }

    #[test]
    fn three_level_commit_propagates_reads_and_writes_to_the_batch() {
        let mut journal = Journal::new(MemoryState::with([
            ("a", 1, "a"),
            ("b", 2, "b"),
            ("c", 3, "c"),
        ]));
        let mut outer = journal.checkpoint();
        assert_eq!(
            journal.read(&StateKey::from("a")).unwrap().value,
            Some(value("a"))
        );
        let mut middle = journal.checkpoint();
        journal.set("b", value("b-next")).unwrap();
        let mut inner = journal.checkpoint();
        assert_eq!(
            journal.read(&StateKey::from("c")).unwrap().revision,
            Revision::new(3)
        );
        journal.set("c", value("c-next")).unwrap();

        journal.commit(&mut inner).unwrap();
        assert_eq!(
            journal.read(&StateKey::from("c")).unwrap().value,
            Some(value("c-next"))
        );
        journal.commit(&mut middle).unwrap();
        assert_eq!(
            journal.read(&StateKey::from("b")).unwrap().value,
            Some(value("b-next"))
        );
        journal.commit(&mut outer).unwrap();

        let batch = journal.transition().unwrap();
        assert_eq!(batch.mutations.len(), 2);
        assert_eq!(batch.mutations[0].key, StateKey::from("b"));
        assert_eq!(batch.mutations[1].key, StateKey::from("c"));
        assert_eq!(batch.reads.len(), 3);
    }

    #[test]
    fn repeated_writes_coalesce_and_final_batch_is_deterministic() {
        let mut journal = Journal::new(MemoryState::with([("a", 1, "old-a"), ("z", 2, "old-z")]));

        journal.set("z", value("z-one")).unwrap();
        journal.set("a", value("a-one")).unwrap();
        journal.set("z", value("z-final")).unwrap();
        journal.set("a", value("a-final")).unwrap();

        let batch = journal.transition().unwrap();
        assert_eq!(batch.mutations.len(), 2);
        assert_eq!(batch.mutations[0].key, StateKey::from("z"));
        assert_eq!(batch.mutations[0].before, Some(value("old-z")));
        assert_eq!(batch.mutations[0].after, Some(value("z-final")));
        assert_eq!(batch.mutations[1].key, StateKey::from("a"));
        assert_eq!(batch.mutations[1].before, Some(value("old-a")));
        assert_eq!(batch.mutations[1].after, Some(value("a-final")));
        assert_eq!(
            batch.reads.keys().cloned().collect::<Vec<_>>(),
            vec![StateKey::from("a"), StateKey::from("z")]
        );
    }

    #[test]
    fn writing_back_the_original_value_emits_no_mutation() {
        let mut journal = Journal::new(MemoryState::with([("key", 1, "original")]));
        journal.set("key", value("temporary")).unwrap();
        journal.set("key", value("original")).unwrap();

        let batch = journal.transition().unwrap();
        assert!(batch.mutations.is_empty());
        assert_eq!(
            batch.reads.get(&StateKey::from("key")),
            Some(&Revision::new(1))
        );
    }

    #[test]
    fn child_commit_preserves_parent_before_value() {
        let mut journal = Journal::new(MemoryState::with([("key", 4, "base")]));
        let mut parent = journal.checkpoint();
        journal.set("key", value("parent")).unwrap();
        let mut child = journal.checkpoint();
        journal.set("key", value("child")).unwrap();
        journal.commit(&mut child).unwrap();
        journal.commit(&mut parent).unwrap();

        let mutation = &journal.transition().unwrap().mutations[0];
        assert_eq!(mutation.expected_revision, Revision::new(4));
        assert_eq!(mutation.before, Some(value("base")));
        assert_eq!(mutation.after, Some(value("child")));
    }

    #[test]
    fn deletion_is_a_read_your_writes_tombstone() {
        let mut journal = Journal::new(MemoryState::with([("key", 9, "value")]));
        journal.delete("key").unwrap();

        assert_eq!(journal.read(&StateKey::from("key")).unwrap().value, None);
        let mutation = &journal.transition().unwrap().mutations[0];
        assert_eq!(mutation.expected_revision, Revision::new(9));
        assert_eq!(mutation.before, Some(value("value")));
        assert_eq!(mutation.after, None);
    }

    #[test]
    fn checkpoint_tokens_are_stack_checked() {
        let mut journal = Journal::new(MemoryState::default());
        let mut parent = journal.checkpoint();
        let mut child = journal.checkpoint();
        assert_eq!(
            journal.commit(&mut parent),
            Err(CheckpointError::NotTopCheckpoint)
        );
        journal.revert(&mut child).unwrap();
        journal.revert(&mut parent).unwrap();
        assert_eq!(
            journal.revert(&mut parent),
            Err(CheckpointError::AlreadyClosed)
        );
    }

    #[test]
    fn rejected_parent_commit_can_be_retried_after_committing_child() {
        let mut journal = Journal::new(MemoryState::default());
        let mut parent = journal.checkpoint();
        journal.set("parent", value("value")).unwrap();
        let mut child = journal.checkpoint();
        journal.set("child", value("value")).unwrap();

        assert_eq!(
            journal.commit(&mut parent),
            Err(CheckpointError::NotTopCheckpoint)
        );
        journal.commit(&mut child).unwrap();
        journal.commit(&mut parent).unwrap();

        let batch = journal.into_transition().unwrap();
        assert_eq!(batch.mutations.len(), 2);
        assert_eq!(batch.mutations[0].key, StateKey::from("parent"));
        assert_eq!(batch.mutations[1].key, StateKey::from("child"));
    }

    #[test]
    fn foreign_checkpoint_rejection_does_not_close_the_handle() {
        let mut first = Journal::new(MemoryState::default());
        let mut second = Journal::new(MemoryState::default());
        let mut checkpoint = first.checkpoint();

        assert_eq!(
            second.revert(&mut checkpoint),
            Err(CheckpointError::ForeignCheckpoint)
        );
        first.revert(&mut checkpoint).unwrap();
        assert!(first.transition().unwrap().mutations.is_empty());
    }
}
