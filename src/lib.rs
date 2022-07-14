#![feature(generic_associated_types)]

use std::{any::TypeId, sync::RwLock};

mod archetype_lookup;
mod entity_manager;
mod queries;
mod sparse_set;
mod world;

pub use queries::*;
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
    fn push_components_to_archetype(self, archetype: &mut Archetype);
    fn append_component_ids(&self, component_ids: &mut Vec<ComponentId>);
}

impl<A: ComponentTrait> ComponentBundleTrait for A {
    fn get_components_and_ids(
        self,
        f: impl FnOnce(&mut [(&mut dyn AnyComponentTrait, ComponentId)]),
    ) {
        f(&mut [(&mut Some(self), A::component_id())])
    }
    fn push_components_to_archetype(self, archetype: &mut Archetype) {
        let channels = archetype.get_corresponding_channels([A::component_id()]);
        get_vec_from_channel::<A>(&mut *archetype.channels[channels[0]].1).push(self);
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
