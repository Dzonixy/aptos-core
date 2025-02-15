// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::task::{ExecutionStatus, ExecutorTask, TransactionOutput};
use aptos_aggregator::{
    delayed_change::DelayedChange,
    delta_change_set::{delta_add, delta_sub, serialize, DeltaOp},
    types::DelayedFieldID,
};
use aptos_mvhashmap::types::TxnIndex;
use aptos_state_view::{StateViewId, TStateView};
use aptos_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    contract_event::ReadWriteEvent,
    event::EventKey,
    executable::ModulePath,
    fee_statement::FeeStatement,
    state_store::{
        state_storage_usage::StateStorageUsage,
        state_value::{StateValue, StateValueMetadataKind},
    },
    transaction::BlockExecutableTransaction as Transaction,
    write_set::{TransactionWrite, WriteOp},
};
use aptos_vm_types::resolver::TExecutorView;
use bytes::Bytes;
use claims::assert_ok;
use move_core_types::{language_storage::TypeTag, value::MoveTypeLayout};
use once_cell::sync::OnceCell;
use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*, proptest, sample::Index};
use proptest_derive::Arbitrary;
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    convert::TryInto,
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

// Should not be possible to overflow or underflow, as each delta is at most 100 in the tests.
// TODO: extend to delta failures.
pub(crate) const STORAGE_AGGREGATOR_VALUE: u128 = 100001;
pub(crate) const MAX_GAS_PER_TXN: u64 = 4;

pub(crate) struct DeltaDataView<K, V> {
    pub(crate) phantom: PhantomData<(K, V)>,
}

impl<K, V> TStateView for DeltaDataView<K, V>
where
    K: PartialOrd + Ord + Send + Sync + Clone + Hash + Eq + ModulePath + 'static,
    V: Debug + Send + Sync + Debug + Clone + TransactionWrite + 'static,
{
    type Key = K;

    /// Gets the state value for a given state key.
    fn get_state_value(&self, _: &K) -> anyhow::Result<Option<StateValue>> {
        Ok(Some(StateValue::new_legacy(
            serialize(&STORAGE_AGGREGATOR_VALUE).into(),
        )))
    }

    fn id(&self) -> StateViewId {
        StateViewId::Miscellaneous
    }

    fn get_usage(&self) -> anyhow::Result<StateStorageUsage> {
        unreachable!();
    }
}

pub(crate) struct EmptyDataView<K, V> {
    pub(crate) phantom: PhantomData<(K, V)>,
}

impl<K, V> TStateView for EmptyDataView<K, V>
where
    K: PartialOrd + Ord + Send + Sync + Clone + Hash + Eq + ModulePath + 'static,
    V: Debug + Send + Sync + Debug + Clone + TransactionWrite + 'static,
{
    type Key = K;

    /// Gets the state value for a given state key.
    fn get_state_value(&self, _: &K) -> anyhow::Result<Option<StateValue>> {
        Ok(None)
    }

    fn id(&self) -> StateViewId {
        StateViewId::Miscellaneous
    }

    fn get_usage(&self) -> anyhow::Result<StateStorageUsage> {
        unreachable!();
    }
}

///////////////////////////////////////////////////////////////////////////
// Generation of transactions
///////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Hash, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) struct KeyType<K: Hash + Clone + Debug + PartialOrd + Ord + Eq>(
    /// Wrapping the types used for testing to add ModulePath trait implementation (below).
    pub K,
    /// The bool field determines for testing purposes, whether the key will be interpreted
    /// as a module access path. In this case, if a module path is both read and written
    /// during parallel execution, ModulePathReadWrite must be returned and the
    /// block execution must fall back to the sequential execution.
    pub bool,
);

impl<K: Hash + Clone + Debug + Eq + PartialOrd + Ord> ModulePath for KeyType<K> {
    fn module_path(&self) -> Option<AccessPath> {
        // Since K is generic, use its hash to assign addresses.
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        let mut hashed_address = vec![1u8; AccountAddress::LENGTH - 8];
        hashed_address.extend_from_slice(&hasher.finish().to_ne_bytes());

        if self.1 {
            Some(AccessPath {
                address: AccountAddress::new(hashed_address.try_into().unwrap()),
                path: b"/foo/b".to_vec(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueType {
    /// Wrapping the types used for testing to add TransactionWrite trait implementation (below).
    bytes: Option<Bytes>,
    metadata: StateValueMetadataKind,
}

impl Arbitrary for ValueType {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        vec(any::<u8>(), 17)
            .prop_map(|mut v| {
                let use_value = v[0] < 128;
                v.resize(16, 0);
                ValueType::new(v, use_value)
            })
            .boxed()
    }
}

impl ValueType {
    /// If use_value is not set, the resulting Value will correspond to a deletion, i.e.
    /// not contain a value (o.w. deletion).
    pub(crate) fn new<V: Into<Vec<u8>> + Debug + Clone + Eq + Send + Sync + Arbitrary>(
        value: V,
        use_value: bool,
    ) -> Self {
        Self {
            bytes: use_value.then(|| {
                let mut v = value.clone().into();
                v.resize(16, 1);
                v.into()
            }),
            metadata: None,
        }
    }

    pub(crate) fn with_len_and_metadata(len: usize, metadata: StateValueMetadataKind) -> Self {
        Self {
            bytes: (len > 0).then_some(vec![100_u8; len].into()),
            metadata,
        }
    }
}

impl TransactionWrite for ValueType {
    fn bytes(&self) -> Option<&Bytes> {
        self.bytes.as_ref()
    }

    fn from_state_value(maybe_state_value: Option<StateValue>) -> Self {
        let (maybe_metadata, maybe_bytes) =
            match maybe_state_value.map(|state_value| state_value.into()) {
                Some((maybe_metadata, bytes)) => (maybe_metadata, Some(bytes)),
                None => (None, None),
            };

        Self {
            bytes: maybe_bytes,
            metadata: maybe_metadata,
        }
    }

    fn as_state_value(&self) -> Option<StateValue> {
        self.extract_raw_bytes().map(|bytes| match &self.metadata {
            Some(metadata) => StateValue::new_with_metadata(bytes, metadata.clone()),
            None => StateValue::new_legacy(bytes),
        })
    }

    fn set_bytes(&mut self, bytes: Bytes) {
        self.bytes = bytes.into();
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TransactionGenParams {
    /// Each transaction's read-set consists of between 1 and read_size-1 many reads.
    read_size: usize,
    /// Each mock execution will produce between 1 and output_size-1 many writes and deltas.
    output_size: usize,
    /// The number of different incarnation behaviors that a mock execution of the transaction
    /// may exhibit. For instance, incarnation_alternatives = 1 corresponds to a "static"
    /// mock execution behavior regardless of the incarnation, while value > 1 may lead to "dynamic",
    /// i.e. different behavior when executing different incarnations of the transaction.
    incarnation_alternatives: usize,
}

#[derive(Arbitrary, Debug, Clone)]
#[proptest(params = "TransactionGenParams")]
pub(crate) struct TransactionGen<V: Into<Vec<u8>> + Arbitrary + Clone + Debug + Eq + 'static> {
    /// Generate keys for possible read-sets of the transaction based on the above parameters.
    #[proptest(
        strategy = "vec(vec(any::<Index>(), 1..params.read_size), params.incarnation_alternatives)"
    )]
    reads: Vec<Vec<Index>>,
    /// Generate keys and values for possible write-sets based on above transaction gen parameters.
    /// Based on how the test is configured, some of these "writes" will convert to deltas.
    #[proptest(
        strategy = "vec(vec((any::<Index>(), any::<V>()), 1..params.output_size), \
		    params.incarnation_alternatives)"
    )]
    modifications: Vec<Vec<(Index, V)>>,
    /// Generate gas for different incarnations of the transactions.
    #[proptest(strategy = "vec(any::<Index>(), params.incarnation_alternatives)")]
    gas: Vec<Index>,
}

/// Describes behavior of a particular incarnation of a mock transaction, as keys to be read,
/// as well as writes, deltas and total execution gas charged for this incarnation. Note that
/// writes, deltas and gas become part of the output directly (as part of the mock execution of
/// a given incarnation), so the output of an incarnation does not depend on the values read, which
/// is a limitation for the testing framework. However, IncarnationBehavior allows different
/// behaviors to be exhibited by different incarnations during parallel execution, which happens
/// first and also records the latest incarnations of each transaction (that is committed).
/// Then we can generate the baseline by sequentially executing the behavior prescribed for
/// those latest incarnations.
#[derive(Clone, Debug)]
pub(crate) struct MockIncarnation<K, V, E> {
    /// A vector of keys to be read during mock incarnation execution.
    pub(crate) reads: Vec<K>,
    /// A vector of keys and corresponding values to be written during mock incarnation execution.
    pub(crate) writes: Vec<(K, V)>,
    /// A vector of keys and corresponding deltas to be produced during mock incarnation execution.
    pub(crate) deltas: Vec<(K, DeltaOp)>,
    // A vector of events.
    pub(crate) events: Vec<E>,
    /// total execution gas to be charged for mock incarnation execution.
    pub(crate) gas: u64,
}

/// A mock transaction that could be used to test the correctness and throughput of the system.
/// To test transaction behavior where reads and writes might be dynamic (depend on previously
/// read values), different read and writes sets are generated and used depending on the incarnation
/// counter value. Each execution of the transaction increments the incarnation counter, and its
/// value determines the index for choosing the read & write sets of the particular execution.
#[derive(Clone, Debug)]
pub(crate) enum MockTransaction<K, V, E> {
    Write {
        /// Incarnation counter, increased during each mock (re-)execution. Allows tracking the final
        /// incarnation for each mock transaction, whose behavior should be reproduced for baseline.
        /// Arc-ed only due to Clone, TODO: clean up the Clone requirement.
        incarnation_counter: Arc<AtomicUsize>,
        /// A vector of mock behaviors prescribed for each incarnation of the transaction, chosen
        /// round robin depending on the incarnation counter value).
        incarnation_behaviors: Vec<MockIncarnation<K, V, E>>,
    },
    /// Skip the execution of trailing transactions.
    SkipRest,
    /// Abort the execution.
    Abort,
}

impl<K, V, E> MockTransaction<K, V, E> {
    pub(crate) fn from_behavior(behavior: MockIncarnation<K, V, E>) -> Self {
        Self::Write {
            incarnation_counter: Arc::new(AtomicUsize::new(0)),
            incarnation_behaviors: vec![behavior],
        }
    }

    pub(crate) fn from_behaviors(behaviors: Vec<MockIncarnation<K, V, E>>) -> Self {
        Self::Write {
            incarnation_counter: Arc::new(AtomicUsize::new(0)),
            incarnation_behaviors: behaviors,
        }
    }
}

impl<
        K: Debug + Hash + Ord + Clone + Send + Sync + ModulePath + 'static,
        V: Clone + Send + Sync + TransactionWrite + 'static,
        E: Debug + Clone + Send + Sync + ReadWriteEvent + 'static,
    > Transaction for MockTransaction<K, V, E>
{
    type Event = E;
    type Identifier = DelayedFieldID;
    type Key = K;
    type Tag = u32;
    type Value = V;
}

// TODO: try and test different strategies.
impl TransactionGenParams {
    pub fn new_dynamic() -> Self {
        TransactionGenParams {
            read_size: 10,
            output_size: 5,
            incarnation_alternatives: 5,
        }
    }
}

impl Default for TransactionGenParams {
    fn default() -> Self {
        TransactionGenParams {
            read_size: 10,
            output_size: 5,
            incarnation_alternatives: 1,
        }
    }
}

// TODO: move generation to separate file.
// TODO: consider adding writes to reads (read-before-write). Similar behavior to the Move-VM
// and may force more testing (since we check read results).
impl<V: Into<Vec<u8>> + Arbitrary + Clone + Debug + Eq + Sync + Send> TransactionGen<V> {
    fn writes_and_deltas_from_gen<K: Clone + Hash + Debug + Eq + Ord>(
        // TODO: disentangle writes and deltas.
        universe: &[K],
        gen: Vec<Vec<(Index, V)>>,
        module_write_fn: &dyn Fn(usize) -> bool,
        delta_fn: &dyn Fn(usize, &V) -> Option<DeltaOp>,
        allow_deletes: bool,
    ) -> Vec<(
        /* writes = */ Vec<(KeyType<K>, ValueType)>,
        /* deltas = */ Vec<(KeyType<K>, DeltaOp)>,
    )> {
        let mut ret = vec![];
        for write_gen in gen.into_iter() {
            let mut keys_modified = BTreeSet::new();
            let mut incarnation_writes = vec![];
            let mut incarnation_deltas = vec![];
            for (idx, value) in write_gen.into_iter() {
                let i = idx.index(universe.len());
                let key = universe[i].clone();
                if !keys_modified.contains(&key) {
                    keys_modified.insert(key.clone());
                    match delta_fn(i, &value) {
                        Some(delta) => incarnation_deltas.push((KeyType(key, false), delta)),
                        None => {
                            // One out of 23 writes will be a deletion
                            let is_deletion = allow_deletes
                                && ValueType::new(value.clone(), true)
                                    .as_u128()
                                    .unwrap()
                                    .unwrap()
                                    % 23
                                    == 0;
                            incarnation_writes.push((
                                KeyType(key, module_write_fn(i)),
                                ValueType::new(value.clone(), !is_deletion),
                            ));
                        },
                    }
                }
            }
            ret.push((incarnation_writes, incarnation_deltas));
        }
        ret
    }

    fn reads_from_gen<K: Clone + Hash + Debug + Eq + Ord>(
        universe: &[K],
        gen: Vec<Vec<Index>>,
        module_read_fn: &dyn Fn(usize) -> bool,
    ) -> Vec<Vec<KeyType<K>>> {
        let mut ret = vec![];
        for read_gen in gen.into_iter() {
            let mut incarnation_reads: Vec<KeyType<K>> = vec![];
            for idx in read_gen.into_iter() {
                let i = idx.index(universe.len());
                let key = universe[i].clone();
                incarnation_reads.push(KeyType(key, module_read_fn(i)));
            }
            ret.push(incarnation_reads);
        }
        ret
    }

    fn gas_from_gen(gas_gen: Vec<Index>) -> Vec<u64> {
        // TODO: generalize gas charging.
        gas_gen
            .into_iter()
            .map(|idx| idx.index(MAX_GAS_PER_TXN as usize + 1) as u64)
            .collect()
    }

    fn new_mock_write_txn<K: Clone + Hash + Debug + Eq + Ord, E: Debug + Clone + ReadWriteEvent>(
        self,
        universe: &[K],
        module_read_fn: &dyn Fn(usize) -> bool,
        module_write_fn: &dyn Fn(usize) -> bool,
        delta_fn: &dyn Fn(usize, &V) -> Option<DeltaOp>,
        allow_deletes: bool,
    ) -> MockTransaction<KeyType<K>, ValueType, E> {
        let reads = Self::reads_from_gen(universe, self.reads, &module_read_fn);
        let gas = Self::gas_from_gen(self.gas);

        let behaviors = Self::writes_and_deltas_from_gen(
            universe,
            self.modifications,
            &module_write_fn,
            &delta_fn,
            allow_deletes,
        )
        .into_iter()
        .zip(reads)
        .zip(gas)
        .map(|(((writes, deltas), reads), gas)| MockIncarnation {
            reads,
            writes,
            deltas,
            events: vec![],
            gas,
        })
        .collect();

        MockTransaction::from_behaviors(behaviors)
    }

    pub(crate) fn materialize<
        K: Clone + Hash + Debug + Eq + Ord,
        E: Send + Sync + Debug + Clone + ReadWriteEvent,
    >(
        self,
        universe: &[K],
        // Are writes and reads module access (same access path).
        module_access: (bool, bool),
    ) -> MockTransaction<KeyType<K>, ValueType, E> {
        let is_module_read = |_| -> bool { module_access.1 };
        let is_module_write = |_| -> bool { module_access.0 };
        let is_delta = |_, _: &V| -> Option<DeltaOp> { None };
        // Module deletion isn't allowed.
        let allow_deletes = !(module_access.0 || module_access.1);

        self.new_mock_write_txn(
            universe,
            &is_module_read,
            &is_module_write,
            &is_delta,
            allow_deletes,
        )
    }

    pub(crate) fn materialize_with_deltas<
        K: Clone + Hash + Debug + Eq + Ord,
        E: Send + Sync + Debug + Clone + ReadWriteEvent,
    >(
        self,
        universe: &[K],
        delta_threshold: usize,
        allow_deletes: bool,
    ) -> MockTransaction<KeyType<K>, ValueType, E> {
        let is_module_read = |_| -> bool { false };
        let is_module_write = |_| -> bool { false };
        let is_delta = |i, v: &V| -> Option<DeltaOp> {
            if i >= delta_threshold {
                let val = ValueType::new(v.clone(), true).as_u128().unwrap().unwrap();
                if val % 10 == 0 {
                    None
                } else if val % 10 < 5 {
                    Some(delta_sub(val % 100, u128::MAX))
                } else {
                    Some(delta_add(val % 100, u128::MAX))
                }
            } else {
                None
            }
        };

        self.new_mock_write_txn(
            universe,
            &is_module_read,
            &is_module_write,
            &is_delta,
            allow_deletes,
        )
    }

    pub(crate) fn materialize_disjoint_module_rw<
        K: Clone + Hash + Debug + Eq + Ord,
        E: Send + Sync + Debug + Clone + ReadWriteEvent,
    >(
        self,
        universe: &[K],
        // keys generated with indices from read_threshold to write_threshold will be
        // treated as module access only in reads. keys generated with indices from
        // write threshold to universe.len() will be treated as module access only in
        // writes. This way there will be module accesses but no intersection.
        read_threshold: usize,
        write_threshold: usize,
    ) -> MockTransaction<KeyType<K>, ValueType, E> {
        assert!(read_threshold < universe.len());
        assert!(write_threshold > read_threshold);
        assert!(write_threshold < universe.len());

        let is_module_read = |i| -> bool { i >= read_threshold && i < write_threshold };
        let is_module_write = |i| -> bool { i >= write_threshold };
        let is_delta = |_, _: &V| -> Option<DeltaOp> { None };

        self.new_mock_write_txn(
            universe,
            &is_module_read,
            &is_module_write,
            &is_delta,
            false, // Module deletion isn't allowed
        )
    }
}

///////////////////////////////////////////////////////////////////////////
// Mock transaction executor implementation.
///////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub(crate) struct MockTask<K, V, E>(PhantomData<(K, V, E)>);

impl<K, V, E> MockTask<K, V, E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<K, V, E> ExecutorTask for MockTask<K, V, E>
where
    K: PartialOrd + Ord + Send + Sync + Clone + Hash + Eq + ModulePath + Debug + 'static,
    V: Send + Sync + Debug + Clone + TransactionWrite + 'static,
    E: Send + Sync + Debug + Clone + ReadWriteEvent + 'static,
{
    type Argument = ();
    type Error = usize;
    type Output = MockOutput<K, V, E>;
    type Txn = MockTransaction<K, V, E>;

    fn init(_argument: Self::Argument) -> Self {
        Self::new()
    }

    fn execute_transaction(
        &self,
        view: &impl TExecutorView<K, u32, MoveTypeLayout, DelayedFieldID>,
        txn: &Self::Txn,
        txn_idx: TxnIndex,
        _materialize_deltas: bool,
    ) -> ExecutionStatus<Self::Output, Self::Error> {
        match txn {
            MockTransaction::Write {
                incarnation_counter,
                incarnation_behaviors,
            } => {
                // Use incarnation counter value as an index to determine the read-
                // and write-sets of the execution. Increment incarnation counter to
                // simulate dynamic behavior when there are multiple possible read-
                // and write-sets (i.e. each are selected round-robin).
                let idx = incarnation_counter.fetch_add(1, Ordering::SeqCst);

                let behavior = &incarnation_behaviors[idx % incarnation_behaviors.len()];

                // Reads
                let mut read_results = vec![];
                for k in behavior.reads.iter() {
                    // TODO: later test errors as well? (by fixing state_view behavior).
                    // TODO: test aggregator reads.
                    match k.module_path() {
                        Some(_) => match view.get_module_bytes(k) {
                            Ok(v) => read_results.push(v.map(Into::into)),
                            Err(_) => read_results.push(None),
                        },
                        None => match view.get_resource_bytes(k, None) {
                            Ok(v) => read_results.push(v.map(Into::into)),
                            Err(_) => read_results.push(None),
                        },
                    }
                }
                ExecutionStatus::Success(MockOutput {
                    writes: behavior.writes.clone(),
                    deltas: behavior.deltas.clone(),
                    events: behavior.events.to_vec(),
                    read_results,
                    materialized_delta_writes: OnceCell::new(),
                    total_gas: behavior.gas,
                })
            },
            MockTransaction::SkipRest => ExecutionStatus::SkipRest(MockOutput::skip_output()),
            MockTransaction::Abort => ExecutionStatus::Abort(txn_idx as usize),
        }
    }
}

#[derive(Debug)]

pub(crate) struct MockOutput<K, V, E> {
    pub(crate) writes: Vec<(K, V)>,
    pub(crate) deltas: Vec<(K, DeltaOp)>,
    pub(crate) events: Vec<E>,
    pub(crate) read_results: Vec<Option<Vec<u8>>>,
    pub(crate) materialized_delta_writes: OnceCell<Vec<(K, WriteOp)>>,
    pub(crate) total_gas: u64,
}

impl<K, V, E> TransactionOutput for MockOutput<K, V, E>
where
    K: PartialOrd + Ord + Send + Sync + Clone + Hash + Eq + ModulePath + Debug + 'static,
    V: Send + Sync + Debug + Clone + TransactionWrite + 'static,
    E: Send + Sync + Debug + Clone + ReadWriteEvent + 'static,
{
    type Txn = MockTransaction<K, V, E>;

    // TODO[agg_v2](tests): Assigning MoveTypeLayout as None for all the writes for now.
    // That means, the resources do not have any DelayedFields embededded in them.
    // Change it to test resources with DelayedFields as well.
    fn resource_write_set(&self) -> BTreeMap<K, (V, Option<Arc<MoveTypeLayout>>)> {
        self.writes
            .iter()
            .filter(|(k, _)| k.module_path().is_none())
            .cloned()
            .map(|(k, v)| (k, (v, None)))
            .collect()
    }

    fn module_write_set(&self) -> BTreeMap<K, V> {
        self.writes
            .iter()
            .filter(|(k, _)| k.module_path().is_some())
            .cloned()
            .collect()
    }

    // Aggregator v1 writes are included in resource_write_set for tests (writes are produced
    // for all keys including ones for v1_aggregators without distinguishing).
    fn aggregator_v1_write_set(&self) -> BTreeMap<K, V> {
        BTreeMap::new()
    }

    fn aggregator_v1_delta_set(&self) -> BTreeMap<K, DeltaOp> {
        self.deltas.iter().cloned().collect()
    }

    fn delayed_field_change_set(
        &self,
    ) -> BTreeMap<
        <Self::Txn as Transaction>::Identifier,
        DelayedChange<<Self::Txn as Transaction>::Identifier>,
    > {
        // TODO[agg_v2](tests): add aggregators V2 to the proptest?
        BTreeMap::new()
    }

    // TODO[agg_v2](tests): Currently, appending None to all events, which means none of the
    // events have aggregators. Test it with aggregators as well.
    fn get_events(&self) -> Vec<(E, Option<MoveTypeLayout>)> {
        self.events.iter().map(|e| (e.clone(), None)).collect()
    }

    fn skip_output() -> Self {
        Self {
            writes: vec![],
            deltas: vec![],
            events: vec![],
            read_results: vec![],
            materialized_delta_writes: OnceCell::new(),
            total_gas: 0,
        }
    }

    fn incorporate_materialized_txn_output(
        &self,
        aggregator_v1_writes: Vec<(<Self::Txn as Transaction>::Key, WriteOp)>,
        _patched_resource_write_set: BTreeMap<
            <Self::Txn as Transaction>::Key,
            <Self::Txn as Transaction>::Value,
        >,
        _patched_events: Vec<<Self::Txn as Transaction>::Event>,
    ) {
        assert_ok!(self.materialized_delta_writes.set(aggregator_v1_writes));
        // TODO[agg_v2](tests): Set the patched resource write set and events. But that requires the function
        // to take &mut self as input
    }

    fn fee_statement(&self) -> FeeStatement {
        // First argument is supposed to be total (not important for the test though).
        // Next two arguments are different kinds of execution gas that are counted
        // towards the block limit. We split the total into two pieces for these arguments.
        // TODO: add variety to generating fee statement based on total gas.
        FeeStatement::new(
            self.total_gas,
            self.total_gas / 2,
            (self.total_gas + 1) / 2,
            0,
            0,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MockEvent {
    key: EventKey,
    sequence_number: u64,
    type_tag: TypeTag,
    event_data: Vec<u8>,
}

impl ReadWriteEvent for MockEvent {
    fn get_event_data(&self) -> (EventKey, u64, &TypeTag, &[u8]) {
        (
            self.key,
            self.sequence_number,
            &self.type_tag,
            &self.event_data,
        )
    }

    fn update_event_data(&mut self, event_data: Vec<u8>) {
        self.event_data = event_data;
    }
}
