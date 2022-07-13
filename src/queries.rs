use std::sync::RwLockReadGuard;

use crate::{
    archetype_lookup::Filter, chained_iterator::ChainedIterator, get_rwlock_from_channel,
    get_vec_from_channel, Archetype, ArchetypeComponentChannel, ComponentId,
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
struct ArchetypeInfo<'a> {
    archetype_index: usize,
    archetype_entities: &'a Vec<usize>,
}
pub struct All<'a, PARAMETERS: QueryParametersTrait> {
    borrow: Vec<(ArchetypeInfo<'a>, PARAMETERS::ResultMut<'a>)>,
}

/*
impl<'a, 'b, PARAMETERS: QueryParametersTrait> IntoIterator for &'a All<'b, PARAMETERS> {
    type Item = <<PARAMETERS::ResultMut<'b> as GetIteratorsTrait>::Iterator<'a> as Iterator>::Item;
    type IntoIter = ChainedIterator<
        std::iter::Map<
            std::slice::Iter<
                'a,
                (
                    ArchetypeInfo<'b>,
                    <PARAMETERS as QueryParametersTrait>::ResultMut<'b>,
                ),
            >,
            fn get_iter<PARAMETERS>(&(ArchetypeInfo, <PARAMETERS as QueryParametersTrait>::ResultMut)),
        >,
    >;
    fn into_iter(self) -> Self::IntoIter {
        let i = ChainedIterator::new(self.borrow.iter().map(get_iter::<PARAMETERS>));

        i
    }
}
*/

pub struct AllBorrow<'a, PARAMETERS: QueryParametersTrait> {
    borrow: Vec<(ArchetypeInfo<'a>, PARAMETERS::Result<'a>)>,
}

impl<'a, PARAMETERS: QueryParametersTrait> All<'a, PARAMETERS> {
    pub fn archetypes_len(&self) -> usize {
        self.borrow.len()
    }
}

pub trait QueryParametersTrait {
    type Result<'a>;
    type ResultMut<'a>;
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

impl<A: ComponentTrait> QueryParametersTrait for &A {
    type Result<'a> = RwLockReadGuard<'a, Vec<A>>;
    type ResultMut<'a> = &'a mut Vec<A>;

    fn get_filters(f: impl FnOnce(&[Filter]) -> Result<(), ECSError>) -> Result<(), ECSError> {
        f(&[Filter {
            filter_type: crate::archetype_lookup::FilterType::With,
            component_id: A::component_id(),
        }])
    }
    fn get_result<'a>(
        archetype_channels: &'a Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::Result<'a>, ECSError> {
        Ok(
            get_rwlock_from_channel::<A>(&*archetype_channels[matching_channels[0].unwrap()].1)
                .read()
                .unwrap(),
        )
    }
    fn get_result_mut<'a>(
        archetype_channels: &'a mut Vec<(ComponentId, Box<dyn ArchetypeComponentChannel>)>,
        matching_channels: &[Option<usize>],
    ) -> Result<Self::ResultMut<'a>, ECSError> {
        Ok(get_vec_from_channel::<A>(
            &mut *archetype_channels[matching_channels[0].unwrap()].1,
        ))
    }
}

impl<PARAMETERS: QueryParametersTrait> QueryTrait for All<'_, PARAMETERS> {
    type Result<'a> = AllBorrow<'a, PARAMETERS>;

    fn get_result<'a>(world: &'a World) -> Result<Self::Result<'a>, ECSError> {
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
                let iter = archetype_lookup.matching_archetype_iter::<1>(filters);
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

/*
pub trait GetIteratorsTrait {
    type Iterator<'a>: Iterator
    where
        Self: 'a;
    type IteratorMut<'a>: Iterator
    where
        Self: 'a;
    fn get_iterator<'a>(&'a self) -> Self::Iterator<'a>;
    fn get_iterator_mut<'a>(&'a mut self) -> Self::IteratorMut<'a>;
    // fn get_component<'a>(&'a self, index: usize) -> <Self::Iterator<'a> as Iterator>::Item;
    // fn get_component_mut<'a>(
    //     &'a mut self,
    //     index: usize,
    // ) -> <Self::IteratorMut<'a> as Iterator>::Item;
}

impl<'b, T: ComponentTrait> GetIteratorsTrait for &'b mut Vec<T> {
    type Iterator<'a> = std::slice::Iter<'a, T> where Self: 'a;
    type IteratorMut<'a> = std::slice::IterMut<'a, T> where Self: 'a;

    fn get_iterator<'a>(&'a self) -> Self::Iterator<'a> {
        self.iter()
    }
    fn get_iterator_mut<'a>(&'a mut self) -> Self::IteratorMut<'a> {
        self.iter_mut()
    }
}
*/
