use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::{
    archetype_lookup::Filter, get_rwlock_from_channel, get_vec_from_channel, query_iterator::*,
    Archetype, ArchetypeComponentChannel, ComponentId,
};

use super::{ComponentTrait, ECSError, World};

pub trait QueryTrait {
    type Result<'a>;
    fn get_result<'a>(world: &'a World) -> Result<Self::Result<'a>, ECSError>;
}

pub trait MutQueryTrait {
    type Result<'a>;
    fn get_result_mut<'a>(world: &'a mut World) -> Result<Self::Result<'a>, ECSError>;
}

/// Info about the [Entity]s in each Archetype
pub struct ArchetypeInfo<'a> {
    archetype_index: usize,
    archetype_entities: &'a Vec<usize>,
}
pub struct All<'a, PARAMETERS: QueryParametersTrait> {
    pub(crate) borrow: Vec<(ArchetypeInfo<'a>, PARAMETERS::ResultMut<'a>)>,
}

pub struct One<'a, PARAMETERS: QueryParametersTrait> {
    borrow: (ArchetypeInfo<'a>, PARAMETERS::ResultMut<'a>),
}

impl<'a, PARAMETERS: QueryParametersTrait> One<'a, PARAMETERS> {
    pub fn get<'b>(
        &'b self,
    ) -> <<PARAMETERS::ResultMut<'a> as GetIteratorsTrait>::Iterator<'b> as Iterator>::Item {
        self.borrow.1.get_iterator().next().unwrap()
    }

    pub fn get_mut<'b>(
        &'b mut self,
    ) -> <<PARAMETERS::ResultMut<'a> as GetIteratorsTrait>::IteratorMut<'b> as Iterator>::Item {
        self.borrow.1.get_iterator_mut().next().unwrap()
    }
}

/*
impl<'a, PARAMETERS: QueryParametersTrait> std::ops::Deref for One<'a, PARAMETERS> {
    type Target =
        <<PARAMETERS::ResultMut<'a> as GetIteratorsTrait>::Iterator<'a> as Iterator>::Item;
    fn deref(&self) -> &Self::Target {
        self.borrow.into_iter().next().unwrap()
    }
}
*/

// I am not a fan of these huge types.

impl<'a, PARAMETERS: QueryParametersTrait> All<'a, PARAMETERS> {
    pub fn archetypes_len(&self) -> usize {
        self.borrow.len()
    }
}

pub trait QueryParameterTrait {
    type Result<'a>: GetIteratorsTrait;
    type ResultMut<'a>: GetIteratorsTrait;
    fn get_component_id() -> ComponentId;
    fn get_result<'a>(
        channel: &'a dyn ArchetypeComponentChannel,
    ) -> Result<Self::Result<'a>, ECSError>;

    fn get_result_mut<'a>(
        channel: &'a mut dyn ArchetypeComponentChannel,
    ) -> Result<Self::ResultMut<'a>, ECSError>;
}

impl<A: ComponentTrait> QueryParameterTrait for &A {
    type Result<'a> = RwLockReadGuard<'a, Vec<A>>;
    type ResultMut<'a> = &'a [A];

    fn get_component_id() -> ComponentId {
        A::component_id()
    }

    fn get_result<'a>(
        channel: &'a dyn ArchetypeComponentChannel,
    ) -> Result<Self::Result<'a>, ECSError> {
        Ok(get_rwlock_from_channel::<A>(channel).read().unwrap())
    }

    fn get_result_mut<'a>(
        channel: &'a mut dyn ArchetypeComponentChannel,
    ) -> Result<Self::ResultMut<'a>, ECSError> {
        Ok(get_vec_from_channel::<A>(channel))
    }
}
impl<A: ComponentTrait> QueryParameterTrait for &mut A {
    type Result<'a> = RwLockWriteGuard<'a, Vec<A>>;
    type ResultMut<'a> = &'a [A];

    fn get_component_id() -> ComponentId {
        A::component_id()
    }

    fn get_result<'a>(
        channel: &'a dyn ArchetypeComponentChannel,
    ) -> Result<Self::Result<'a>, ECSError> {
        Ok(get_rwlock_from_channel::<A>(channel).write().unwrap())
    }

    fn get_result_mut<'a>(
        channel: &'a mut dyn ArchetypeComponentChannel,
    ) -> Result<Self::ResultMut<'a>, ECSError> {
        Ok(get_vec_from_channel::<A>(channel))
    }
}

pub trait QueryParametersTrait {
    type Result<'a>: GetIteratorsTrait;
    type ResultMut<'a>: GetIteratorsTrait;
    const FILTER_COUNT: usize;

    fn get_filters(f: impl FnOnce(&[Filter]) -> Result<(), ECSError>) -> Result<(), ECSError>;
    fn get_result<'a>(
        archetype_channels: &'a Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::Result<'a>, ECSError>;
    fn get_result_mut<'a>(
        archetype_channels: &'a mut Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::ResultMut<'a>, ECSError>;
}

impl<A: QueryParameterTrait> QueryParametersTrait for A {
    type Result<'a> = A::Result<'a>;
    type ResultMut<'a> = A::ResultMut<'a>;
    const FILTER_COUNT: usize = 1;

    fn get_filters(f: impl FnOnce(&[Filter]) -> Result<(), ECSError>) -> Result<(), ECSError> {
        f(&[Filter {
            filter_type: crate::archetype_lookup::FilterType::With,
            component_id: A::get_component_id(),
        }])
    }
    fn get_result<'a>(
        archetype_channels: &'a Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::Result<'a>, ECSError> {
        A::get_result(&*archetype_channels[matching_channels[0].unwrap()].1)
    }
    fn get_result_mut<'a>(
        archetype_channels: &'a mut Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::ResultMut<'a>, ECSError> {
        A::get_result_mut(&mut *archetype_channels[matching_channels[0].unwrap()].1)
    }
}

impl<A: QueryParameterTrait, B: QueryParameterTrait> QueryParametersTrait for (A, B) {
    type Result<'a> = (A::Result<'a>, B::Result<'a>);
    type ResultMut<'a> = (A::ResultMut<'a>, B::ResultMut<'a>);
    const FILTER_COUNT: usize = 2;

    fn get_filters(f: impl FnOnce(&[Filter]) -> Result<(), ECSError>) -> Result<(), ECSError> {
        f(&[
            Filter {
                filter_type: crate::archetype_lookup::FilterType::With,
                component_id: A::get_component_id(),
            },
            Filter {
                filter_type: crate::archetype_lookup::FilterType::With,
                component_id: B::get_component_id(),
            },
        ])
    }
    fn get_result<'a>(
        archetype_channels: &'a Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::Result<'a>, ECSError> {
        Ok((
            A::get_result(&*archetype_channels[matching_channels[0].unwrap()].1)?,
            B::get_result(&*archetype_channels[matching_channels[1].unwrap()].1)?,
        ))
    }
    fn get_result_mut<'a>(
        archetype_channels: &'a mut Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::ResultMut<'a>, ECSError> {
        println!("MATCHING CHANNELS: {:?}", matching_channels);
        // How can both channels be indexed safely?
        todo!()
        /*
        Ok((
            A::get_result_mut(&mut *archetype_channels[matching_channels[0].unwrap()].1)?,
            B::get_result_mut(&mut *archetype_channels[matching_channels[1].unwrap()].1)?,
        ))
        */
    }
}

impl<PARAMETERS: QueryParametersTrait> QueryTrait for All<'_, PARAMETERS> {
    type Result<'a> = AllBorrow<'a, PARAMETERS>;

    fn get_result<'a>(world: &'a World) -> Result<Self::Result<'a>, ECSError> {
        // I'd like to figure out how to avoid this `Vec::new()`
        // But probably it can't be done without unsafe.
        let mut borrow = Vec::new();
        let World {
            archetypes,
            archetype_lookup,
            ..
        } = world;
        {
            let borrow = &mut borrow;
            let mut archetypes: &[Archetype] = archetypes;

            PARAMETERS::get_filters(move |filters| {
                // Todo: I need to figure out how to get rid of this 8. It's incorrect.
                let iter = archetype_lookup.matching_archetype_iter::<8>(filters);
                for (archetype_index, matching_channels) in iter {
                    // We must use splitting borrows to appease the borrow checker.
                    // Fortunately the indices returned by `matching_archetype_iter` increase.
                    let (left, right) = archetypes.split_at(archetype_index + 1);
                    archetypes = right;
                    let Archetype {
                        channels,
                        entity_indices,
                    } = &left[0];
                    let result = PARAMETERS::get_result(channels, &matching_channels)?;
                    borrow.push((
                        ArchetypeInfo {
                            archetype_entities: entity_indices,
                            archetype_index: archetype_index,
                        },
                        result,
                    ))
                }
                Ok(())
            })?
        }

        Ok(AllBorrow { borrow })
    }
}

impl<PARAMETERS: QueryParametersTrait> MutQueryTrait for All<'_, PARAMETERS> {
    type Result<'a> = All<'a, PARAMETERS>;

    fn get_result_mut<'a>(world: &'a mut World) -> Result<Self::Result<'a>, ECSError> {
        // I'd like to figure out how to avoid this `Vec::new()`
        // But probably it can't be done without unsafe.

        let mut borrow = Vec::new();
        let World {
            archetypes,
            archetype_lookup,
            ..
        } = world;
        {
            let borrow = &mut borrow;
            let mut archetypes: &mut [Archetype] = archetypes;

            PARAMETERS::get_filters(move |filters| {
                let iter = archetype_lookup.matching_archetype_iter::<1>(filters);
                for (archetype_index, matching_channels) in iter {
                    // We must use splitting borrows to appease the borrow checker.
                    // Fortunately the indices returned by `matching_archetype_iter` increase.
                    let (left, right) = archetypes.split_at_mut(archetype_index + 1);
                    archetypes = right;
                    let Archetype {
                        channels,
                        entity_indices,
                    } = &mut left[0];
                    let result = PARAMETERS::get_result_mut(channels, &matching_channels)?;
                    borrow.push((
                        ArchetypeInfo {
                            archetype_entities: entity_indices,
                            archetype_index: archetype_index,
                        },
                        result,
                    ))
                }
                Ok(())
            })?
        }

        Ok(All { borrow })
    }
}

impl<PARAMETERS: QueryParametersTrait> MutQueryTrait for One<'_, PARAMETERS> {
    type Result<'a> = One<'a, PARAMETERS>;

    fn get_result_mut<'a>(world: &'a mut World) -> Result<Self::Result<'a>, ECSError> {
        // I'd like to figure out how to avoid this `Vec::new()`
        // But probably it can't be done without unsafe.

        let mut borrow = Err(ECSError::NoMatchingComponent);
        let World {
            archetypes,
            archetype_lookup,
            ..
        } = world;
        {
            let borrow = &mut borrow;
            let mut archetypes: &mut [Archetype] = archetypes;

            PARAMETERS::get_filters(move |filters| {
                let iter = archetype_lookup.matching_archetype_iter::<1>(filters);
                for (archetype_index, matching_channels) in iter {
                    // We must use splitting borrows to appease the borrow checker.
                    // Fortunately the indices returned by `matching_archetype_iter` increase.
                    let (left, right) = archetypes.split_at_mut(archetype_index + 1);
                    archetypes = right;
                    let Archetype {
                        channels,
                        entity_indices,
                    } = &mut left[0];
                    let result = PARAMETERS::get_result_mut(channels, &matching_channels)?;
                    if !entity_indices.is_empty() {
                        *borrow = Ok((
                            ArchetypeInfo {
                                archetype_entities: entity_indices,
                                archetype_index: archetype_index,
                            },
                            result,
                        ));
                        break;
                    }
                }
                Ok(())
            })?
        }

        Ok(One { borrow: borrow? })
    }
}
