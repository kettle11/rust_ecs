use crate::*;

pub trait ArchetypeComponentChannel {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn migrate(&mut self, other: &mut dyn ArchetypeComponentChannel, index: usize);
    fn swap_remove(&mut self, index: usize);
    fn push(&mut self, component: &mut dyn AnyComponentTrait);
}

impl<COMPONENT: ComponentTrait> ArchetypeComponentChannel for RwLock<Vec<COMPONENT>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn migrate(&mut self, other: &mut dyn ArchetypeComponentChannel, index: usize) {
        let component = self.get_mut().unwrap().swap_remove(index);
        other
            .as_any_mut()
            .downcast_mut::<RwLock<Vec<COMPONENT>>>()
            .unwrap()
            .get_mut()
            .unwrap()
            .push(component)
    }
    fn swap_remove(&mut self, index: usize) {
        self.get_mut().unwrap().swap_remove(index);
    }
    fn push(&mut self, component: &mut dyn AnyComponentTrait) {
        self.get_mut().unwrap().push(
            component
                .as_any_mut()
                .downcast_mut::<Option<COMPONENT>>()
                .unwrap()
                .take()
                .unwrap(),
        )
    }
}

pub struct Archetype {
    pub(crate) entity_indices: Vec<usize>,
    pub(crate) channels: Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
}

impl Archetype {
    fn new() -> Self {
        Self {
            entity_indices: Vec::new(),
            channels: Vec::new(),
        }
    }
    fn remove_entity(
        &mut self,
        entity_manager: &mut entity_manager::EntityManager,
        entity_index_in_archetype: usize,
    ) {
        let to_be_swap_removed = *self.entity_indices.last().unwrap();
        entity_manager
            .update_entity_index_in_archetype(to_be_swap_removed, entity_index_in_archetype);
        for channel in self.channels.iter_mut() {
            channel.1.swap_remove(entity_index_in_archetype);
        }
    }

    pub fn get_corresponding_channels<const COUNT: usize>(
        &self,
        component_ids: [ComponentId; COUNT],
    ) -> [usize; COUNT] {
        let mut out = [0; COUNT];
        for (component_id, out) in component_ids.iter().zip(out.iter_mut()) {
            for (i, channel) in self.channels.iter().enumerate() {
                if channel.0 == *component_id {
                    *out = i
                }
            }
        }
        out
    }

    /// Moves an [Entity]'s components from this [Archetype] to another [Archetype]
    pub fn migrate_entity_components(&mut self, other: &mut Archetype, entity_index: usize) {
        for channel in self.channels.iter_mut() {
            for other_channel in other.channels.iter_mut() {
                if channel.0 == other_channel.0 {
                    channel.1.migrate(&mut *other_channel.1, entity_index);
                    break;
                }
            }
        }
    }
}

pub(crate) fn get_vec_from_channel<COMPONENT: ComponentTrait>(
    channel: &mut dyn ArchetypeComponentChannel,
) -> &mut Vec<COMPONENT> {
    channel
        .as_any_mut()
        .downcast_mut::<RwLock<Vec<COMPONENT>>>()
        .unwrap()
        .get_mut()
        .unwrap()
}

pub(crate) fn get_rwlock_from_channel<COMPONENT: ComponentTrait>(
    channel: &dyn ArchetypeComponentChannel,
) -> &RwLock<Vec<COMPONENT>> {
    channel
        .as_any()
        .downcast_ref::<RwLock<Vec<COMPONENT>>>()
        .unwrap()
}

pub struct World {
    pub(crate) entity_manager: entity_manager::EntityManager,
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) archetype_lookup: archetype_lookup::ArchetypeLookup,
    /// A scratch-buffer for ComponentIds
    pub(crate) component_ids_temp: Vec<ComponentId>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_manager: entity_manager::EntityManager::new(),
            archetypes: Vec::new(),
            archetype_lookup: archetype_lookup::ArchetypeLookup::new(),
            component_ids_temp: Vec::new(),
        }
    }

    pub fn spawn<COMPONENTS: ComponentBundleTrait>(&mut self, components: COMPONENTS) -> Entity {
        let mut entity = Entity::from_index_and_generation(0, 0);
        components.get_components_and_ids(|v| {
            entity = self.spawn_inner(v);
        });
        entity
    }

    fn spawn_inner(
        &mut self,
        components_and_ids: &mut [(&mut dyn AnyComponentTrait, ComponentId)],
    ) -> Entity {
        components_and_ids.sort_by_key(|v| v.1);
        self.component_ids_temp.clear();
        self.component_ids_temp
            .extend(components_and_ids.iter().map(|v| v.1));

        let len = self.component_ids_temp.len();
        self.component_ids_temp.dedup();
        assert_eq!(
            len,
            self.component_ids_temp.len(),
            "Cannot spawn `Entity`s with multiple of the same component"
        );

        let archetype_index = if let Some(archetype_index) = self
            .archetype_lookup
            .get_exact_archetype(&self.component_ids_temp)
        {
            archetype_index
        } else {
            // Create a new archetype
            let mut new_archetype = Archetype::new();
            for (component, component_id) in components_and_ids.iter() {
                new_archetype
                    .channels
                    .push((*component_id, component.new_archetype_channel()));
            }
            let archetype_index = self.archetypes.len();
            self.archetypes.push(new_archetype);
            self.archetype_lookup
                .new_archetype(&self.component_ids_temp);
            archetype_index
        };

        let archetype = &mut self.archetypes[archetype_index];
        for ((component, _), (_, channel)) in components_and_ids
            .iter_mut()
            .zip(archetype.channels.iter_mut())
        {
            channel.push(*component)
        }

        let entity = self.entity_manager.new_entity(EntityLocation {
            storage_index: archetype_index,
            index_within_storage: archetype.entity_indices.len(),
        });
        archetype.entity_indices.push(entity.index);
        entity
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), ECSError> {
        let entity_location = self.entity_manager.get_entity_location(entity)?;
        self.archetypes[entity_location.storage_index].remove_entity(
            &mut self.entity_manager,
            entity_location.index_within_storage,
        );
        Ok(())
    }

    pub fn add_components<COMPONENTS: ComponentBundleTrait>(
        &mut self,
        entity: Entity,
        components: COMPONENTS,
    ) -> Result<(), ECSError> {
        let entity_location = self.entity_manager.get_entity_location(entity)?;
        let archetype = &self.archetypes[entity_location.storage_index];

        // Todo: Merge the added [ComponentID]s with the existing [Archetype]'s IDs.
        //  Then find  or create the new [Archetype] to migrate this [Entity] to.
        todo!()
    }

    pub fn remove_components<COMPONENTS: ComponentBundleTrait>(
        &mut self,
        entity: Entity,
    ) -> Result<COMPONENTS, ECSError> {
        let entity_location = self.entity_manager.get_entity_location(entity)?;
        // Todo: Remove the components and store them in the component bundle.
        todo!()
    }

    /// Move all components and [Entity]s from `other` into this [World].
    pub fn append(&mut self, other: &mut World) {
        todo!()
    }

    pub fn query<'a, QUERY: QueryTrait>(&'a self) -> QUERY::Result<'a> {
        self.try_query::<QUERY>().unwrap()
    }
    //
    pub fn try_query<'a, QUERY: QueryTrait>(&'a self) -> Result<QUERY::Result<'a>, ECSError> {
        QUERY::get_result(self)
    }

    /// Faster than [query] because it can avoid exclusive borrowing checks.
    pub fn query_mut<'a, QUERY: MutQueryTrait>(&'a mut self) -> QUERY::Result<'a> {
        QUERY::get_result_mut(self).unwrap()
    }
}

impl Clone for World {
    fn clone(&self) -> Self {
        todo!()
    }
}
