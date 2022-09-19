#![feature(generic_associated_types)]

use std::{any::TypeId, sync::RwLock};

mod archetype_lookup;
mod entity_manager;

#[macro_use]
mod queries;

#[macro_use]
mod query_iterator;

#[macro_use]
mod multi_iterator;

mod sparse_set;
mod world;

pub use multi_iterator::*;
pub use queries::*;
pub use query_iterator::*;
pub use world::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Entity {
    index: usize,
    generation: u32,
}

impl Entity {
    pub fn from_index_and_generation(index: usize, generation: u32) -> Self {
        Self { index, generation }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct ComponentId {
    type_id: TypeId,
}

#[derive(Debug, Clone, Copy)]
pub enum ECSError {
    NoMatchingComponent,
    NoMatchingEntity,
    EntityNoLongerExists,
}

#[derive(Clone, Copy)]
pub struct EntityLocation {
    storage_index: usize,
    index_within_storage: usize,
}

pub trait ComponentTrait: 'static + Send + Sync + Sized {
    fn clone_vec(data: &[Self]) -> Option<Vec<Self>>;
    fn make_vec() -> Vec<Self> {
        Vec::new()
    }
    fn component_id() -> ComponentId {
        ComponentId {
            type_id: std::any::TypeId::of::<Self>(),
        }
    }
}

pub trait ComponentBundleTrait {
    fn get_components_and_ids(
        self,
        f: impl FnOnce(&mut [(&mut dyn AnyComponentTrait, ComponentId)]),
    );
    fn append_component_ids(&self, component_ids: &mut Vec<ComponentId>);
}

impl<A: ComponentTrait> ComponentBundleTrait for A {
    fn get_components_and_ids(
        self,
        f: impl FnOnce(&mut [(&mut dyn AnyComponentTrait, ComponentId)]),
    ) {
        f(&mut [(&mut Some(self), A::component_id())])
    }

    fn append_component_ids(&self, component_ids: &mut Vec<ComponentId>) {
        component_ids.push(A::component_id())
    }
}

impl<A: ComponentTrait, B: ComponentTrait> ComponentBundleTrait for (A, B) {
    fn get_components_and_ids(
        self,
        f: impl FnOnce(&mut [(&mut dyn AnyComponentTrait, ComponentId)]),
    ) {
        f(&mut [
            (&mut Some(self.0), A::component_id()),
            (&mut Some(self.1), B::component_id()),
        ])
    }

    fn append_component_ids(&self, component_ids: &mut Vec<ComponentId>) {
        component_ids.push(A::component_id())
    }
}

pub trait AnyComponentTrait: std::any::Any {
    fn new_archetype_channel(&self) -> Box<dyn ArchetypeComponentChannel>;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<COMPONENT: ComponentTrait> AnyComponentTrait for Option<COMPONENT> {
    fn new_archetype_channel(&self) -> Box<dyn ArchetypeComponentChannel> {
        Box::new(RwLock::new(Vec::<COMPONENT>::new()))
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
macro_rules! tuple_impls {
    ( $count: tt, $( ($index: tt, $tuple:ident) ),*) => {
        // system_tuple_impls! { $count, $( ($index, $tuple) ),*}
        // component_bundle_tuple_impls! { $count, $( ($index, $tuple) ),*}
         multi_iterator_impl! { $count, $( ($index, $tuple) ),*}
        // query_impls! { $count, $( ($index, $tuple) ),*}
         query_iterator_impls! { $count, $( ($index, $tuple) ),*}
        // singleton_impls! { $count, $( ($index, $tuple) ),*}
    };
}

tuple_impls! {0,}
tuple_impls! { 1, (0, A) }
tuple_impls! { 2, (0, A), (1, B) }
tuple_impls! { 3, (0, A), (1, B), (2, C) }
tuple_impls! { 4, (0, A), (1, B), (2, C), (3, D)}
tuple_impls! { 5, (0, A), (1, B), (2, C), (3, D), (4, E)}
tuple_impls! { 6, (0, A), (1, B), (2, C), (3, D), (4, E), (5, F)}
tuple_impls! { 7, (0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G)}
tuple_impls! { 8, (0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H)}
tuple_impls! { 9, (0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I)}
tuple_impls! { 10, (0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I), (9, J)}
tuple_impls! { 11, (0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I), (9, J), (10, K)}
tuple_impls! { 12, (0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I), (9, J), (10, K), (11, L)}
