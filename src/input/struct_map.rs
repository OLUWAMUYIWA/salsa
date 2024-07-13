use std::{ops::DerefMut, sync::Arc};

use dashmap::mapref::one::RefMut;

use crate::{alloc::Alloc, hash::FxDashMap, id::FromId, Id};

use super::{Configuration, Value};

/// Maps an input id to its data.
///
/// Input structs can only be mutated by a call to a setter with an `&mut`
/// reference to the database, and therefore cannot be mutated during a tracked
/// function or in parallel.
///
/// However for on-demand inputs to work the fields must be able to be set via
/// a shared reference, so some locking is required.
///
/// Altogether this makes the implementation somewhat simpler than tracked
/// structs.
pub(crate) struct StructMap<C>
where
    C: Configuration,
{
    map: Arc<FxDashMap<Id, Alloc<Value<C>>>>,
}

impl<C: Configuration> Clone for StructMap<C> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

impl<C> StructMap<C>
where
    C: Configuration,
{
    pub fn new() -> Self {
        Self {
            map: Arc::new(FxDashMap::default()),
        }
    }

    /// Insert the given tracked struct value into the map.
    ///
    /// # Panics
    ///
    /// * If value with same `value.id` is already present in the map.
    /// * If value not created in current revision.
    pub fn insert<'db>(&'db self, value: Value<C>) -> C::Id {
        let id = value.id;
        let boxed_value = Alloc::new(value);
        let old_value = self.map.insert(id, boxed_value);
        assert!(old_value.is_none()); // ...strictly speaking we probably need to abort here
        C::Id::from_id(id)
    }

    /// Get immutable access to the data for `id` -- this holds a write lock for the duration
    /// of the returned value.
    ///
    /// # Panics
    ///
    /// * If the value is not present in the map.
    /// * If the value is already updated in this revision.
    pub fn get<'db>(&'db self, id: Id) -> &'db Value<C> {
        /// More limited wrapper around transmute that copies lifetime from `a` to `b`.
        ///
        /// # Safety condition
        ///
        /// `b` must be owned by `a`
        unsafe fn transmute_lifetime<'a, 'b, A, B>(_a: &'a A, b: &'b B) -> &'a B {
            std::mem::transmute(b)
        }
        unsafe { transmute_lifetime(self, self.map.get(&id).unwrap().as_ref()) }
    }

    /// Get mutable access to the data for `id` -- this holds a write lock for the duration
    /// of the returned value.
    ///
    /// # Panics
    ///
    /// * If the value is not present in the map.
    /// * If the value is already updated in this revision.
    pub fn update<'db>(&'db mut self, id: Id) -> impl DerefMut<Target = Value<C>> + 'db {
        RefMut::map(self.map.get_mut(&id).unwrap(), |v| unsafe { v.as_mut() })
    }

    /// Remove the entry for `id` from the map.
    pub fn delete(&mut self, id: Id) -> Option<Value<C>> {
        if let Some((_, data)) = self.map.remove(&id) {
            Some(data.into_inner())
        } else {
            None
        }
    }
}