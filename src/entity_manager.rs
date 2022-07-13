use super::{ECSError, Entity, EntityLocation};

pub(crate) struct EntityManager {
    free_entities: Vec<usize>,
    entity_index_to_generation_and_location: Vec<(u32, EntityLocation)>,
}

impl EntityManager {
    pub(crate) fn new() -> Self {
        Self {
            free_entities: Vec::new(),
            entity_index_to_generation_and_location: Vec::new(),
        }
    }
    pub(crate) fn new_entity(&mut self, entity_location: EntityLocation) -> Entity {
        // Reuse an old Entity's index if possible, otherwise create a new index.
        if let Some(index) = self.free_entities.pop() {
            // The generation was already incremented when the former Entity was despawned.
            let (generation, location) =
                &mut self.entity_index_to_generation_and_location[index as usize];
            *location = entity_location;
            Entity {
                index,
                generation: *generation,
            }
        } else {
            self.entity_index_to_generation_and_location
                .push((0, entity_location));
            Entity {
                index: self.entity_index_to_generation_and_location.len() - 1,
                generation: 0,
            }
        }
    }

    pub(crate) fn get_entity_location(&self, entity: Entity) -> Result<EntityLocation, ECSError> {
        if let Some(&(generation, entity_location)) = self
            .entity_index_to_generation_and_location
            .get(entity.index)
        {
            if generation == entity.generation {
                Ok(entity_location)
            } else {
                Err(ECSError::EntityNoLongerExists)
            }
        } else {
            Err(ECSError::NoMatchingEntity)
        }
    }

    pub(crate) fn despawn_entity(&mut self, entity: Entity) {
        if let Some((generation, _entity_location)) = self
            .entity_index_to_generation_and_location
            .get_mut(entity.index)
        {
            if *generation == entity.generation {
                // Increment the generation so that further attempts to reference this Entity will be invalid.
                *generation += 1;
                self.free_entities.push(entity.index);
            }
        }
    }

    pub(crate) fn update_entity_index_in_archetype(
        &mut self,
        entity_index: usize,
        entity_index_in_archetype: usize,
    ) {
        self.entity_index_to_generation_and_location[entity_index]
            .1
            .index_within_storage = entity_index_in_archetype;
    }
}
