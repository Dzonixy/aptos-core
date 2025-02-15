// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{types::MVModulesOutput, utils::module_hash};
use aptos_aggregator::types::DelayedFieldValue;
use aptos_crypto::hash::HashValue;
use aptos_types::{
    executable::{Executable, ExecutableDescriptor, ModulePath},
    write_set::TransactionWrite,
};
use move_core_types::value::MoveTypeLayout;
use std::{cell::RefCell, collections::HashMap, hash::Hash, sync::Arc};

/// UnsyncMap is designed to mimic the functionality of MVHashMap for sequential execution.
/// In this case only the latest recorded version is relevant, simplifying the implementation.
/// The functionality also includes Executable caching based on the hash of ExecutableDescriptor
/// (i.e. module hash for modules published during the latest block - not at storage version).
pub struct UnsyncMap<K: ModulePath, V: TransactionWrite, X: Executable, I: Copy> {
    // Only use Arc to provide unified interfaces with the MVHashMap / concurrent setting. This
    // simplifies the trait-based integration for executable caching. TODO: better representation.
    // Optional hash can store the hash of the module to avoid re-computations.
    map: RefCell<HashMap<K, (Arc<V>, Option<HashValue>, Option<Arc<MoveTypeLayout>>)>>,
    executable_cache: RefCell<HashMap<HashValue, Arc<X>>>,
    executable_bytes: RefCell<usize>,
    delayed_field_map: RefCell<HashMap<I, DelayedFieldValue>>,
}

impl<
        K: ModulePath + Hash + Clone + Eq,
        V: TransactionWrite,
        X: Executable,
        I: Hash + Clone + Copy + Eq,
    > Default for UnsyncMap<K, V, X, I>
{
    fn default() -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
            executable_cache: RefCell::new(HashMap::new()),
            executable_bytes: RefCell::new(0),
            delayed_field_map: RefCell::new(HashMap::new()),
        }
    }
}

impl<
        K: ModulePath + Hash + Clone + Eq,
        V: TransactionWrite,
        X: Executable,
        I: Hash + Clone + Copy + Eq,
    > UnsyncMap<K, V, X, I>
{
    pub fn new() -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
            executable_cache: RefCell::new(HashMap::new()),
            executable_bytes: RefCell::new(0),
            delayed_field_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn fetch_data(&self, key: &K) -> Option<(Arc<V>, Option<Arc<MoveTypeLayout>>)> {
        self.map
            .borrow()
            .get(key)
            .map(|entry| (entry.0.clone(), entry.2.clone()))
    }

    pub fn fetch_module(&self, key: &K) -> Option<MVModulesOutput<V, X>> {
        use MVModulesOutput::*;
        debug_assert!(key.module_path().is_some());

        self.map.borrow_mut().get_mut(key).map(|entry| {
            let hash = entry.1.get_or_insert(module_hash(entry.0.as_ref()));

            self.executable_cache.borrow().get(hash).map_or_else(
                || Module((entry.0.clone(), *hash)),
                |x| Executable((x.clone(), ExecutableDescriptor::Published(*hash))),
            )
        })
    }

    pub fn fetch_delayed_field(&self, id: &I) -> Option<DelayedFieldValue> {
        self.delayed_field_map.borrow().get(id).cloned()
    }

    pub fn write(&self, key: K, value: V, layout: Option<Arc<MoveTypeLayout>>) {
        self.map
            .borrow_mut()
            .insert(key, (Arc::new(value), None, layout));
    }

    /// We return false if the executable was already stored, as this isn't supposed to happen
    /// during sequential execution (and the caller may choose to e.g. log a message).
    /// Versioned modules storage does not cache executables at storage version, hence directly
    /// the descriptor hash in ExecutableDescriptor::Published is provided.
    pub fn store_executable(&self, descriptor_hash: HashValue, executable: X) -> bool {
        let size = executable.size_bytes();
        if self
            .executable_cache
            .borrow_mut()
            .insert(descriptor_hash, Arc::new(executable))
            .is_some()
        {
            *self.executable_bytes.borrow_mut() += size;
            true
        } else {
            false
        }
    }

    pub fn executable_size(&self) -> usize {
        *self.executable_bytes.borrow()
    }

    pub fn write_delayed_field(&self, id: I, value: DelayedFieldValue) {
        self.delayed_field_map.borrow_mut().insert(id, value);
    }
}
