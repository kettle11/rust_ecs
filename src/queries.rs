use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::{
    archetype_lookup::Filter, get_rwlock_from_channel, get_vec_from_channel, Archetype,
    ArchetypeComponentChannel, ComponentId,
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
    borrow: Vec<(ArchetypeInfo<'a>, PARAMETERS::ResultMut<'a>)>,
}

pub struct One<'a, PARAMETERS: QueryParametersTrait> {
    borrow: (ArchetypeInfo<'a>, PARAMETERS::ResultMut<'a>),
}

// I am not a fan of these huge types.

pub type QueryBorrowIter<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::Iter<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait<'b>>::Iterator,
    fn(
        &'b (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait<'b>>::Iterator,
>;

pub type QueryBorrowIterMut<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::IterMut<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait<'b>>::IteratorMut,
    fn(
        &'b mut (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait<'b>>::IteratorMut,
>;

pub type QueryIter<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::Iter<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait<'b>>::Iterator,
    fn(
        &'b (
            ArchetypeInfo,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait<'b>>::Iterator,
>;

pub type QueryIterMut<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::IterMut<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait<'b>>::IteratorMut,
    fn(
        &'b mut (
            ArchetypeInfo,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait<'b>>::IteratorMut,
>;

impl<'a, 'b, PARAMETERS: QueryParametersTrait> IntoIterator for &'b All<'a, PARAMETERS> {
    type Item = <QueryIter<'a, 'b, PARAMETERS> as Iterator>::Item;
    type IntoIter = QueryIter<'a, 'b, PARAMETERS>;
    fn into_iter(self) -> Self::IntoIter {
        self.borrow.iter().flat_map(|v| v.1.get_iterator())
    }
}

impl<'a, 'b, PARAMETERS: QueryParametersTrait> IntoIterator for &'b mut All<'a, PARAMETERS> {
    type Item = <QueryIterMut<'a, 'b, PARAMETERS> as Iterator>::Item;
    type IntoIter = QueryIterMut<'a, 'b, PARAMETERS>;
    fn into_iter(self) -> Self::IntoIter {
        self.borrow.iter_mut().flat_map(|v| v.1.get_iterator_mut())
    }
}
// The type of iterator returned is relatively complex.
// I'm not even sure if it can be expressed in a way to implement IntoIterator.
// Is there a better approach?
impl<'a, PARAMETERS: QueryParametersTrait> All<'a, PARAMETERS> {
    pub fn iter<'b>(&'b self) -> QueryIter<'a, 'b, PARAMETERS> {
        self.into_iter()
    }

    pub fn iter_mut<'b>(&'b mut self) -> QueryIterMut<'a, 'b, PARAMETERS> {
        self.into_iter()
    }
}

pub struct AllBorrow<'a, PARAMETERS: QueryParametersTrait> {
    borrow: Vec<(ArchetypeInfo<'a>, PARAMETERS::Result<'a>)>,
}

impl<'a, 'b, PARAMETERS: QueryParametersTrait> IntoIterator for &'b AllBorrow<'a, PARAMETERS> {
    type Item = <QueryBorrowIter<'a, 'b, PARAMETERS> as Iterator>::Item;
    type IntoIter = QueryBorrowIter<'a, 'b, PARAMETERS>;
    fn into_iter(self) -> Self::IntoIter {
        self.borrow.iter().flat_map(|v| v.1.get_iterator())
    }
}

impl<'a, 'b, PARAMETERS: QueryParametersTrait> IntoIterator for &'b mut AllBorrow<'a, PARAMETERS> {
    type Item = <QueryBorrowIterMut<'a, 'b, PARAMETERS> as Iterator>::Item;
    type IntoIter = QueryBorrowIterMut<'a, 'b, PARAMETERS>;
    fn into_iter(self) -> Self::IntoIter {
        self.borrow.iter_mut().flat_map(|v| v.1.get_iterator_mut())
    }
}

impl<'a, PARAMETERS: QueryParametersTrait> AllBorrow<'a, PARAMETERS> {
    pub fn iter<'b>(&'b self) -> QueryBorrowIter<'a, 'b, PARAMETERS> {
        self.into_iter()
    }

    pub fn iter_mut<'b>(&'b mut self) -> QueryBorrowIterMut<'a, 'b, PARAMETERS> {
        self.into_iter()
    }
}

impl<'a, PARAMETERS: QueryParametersTrait> All<'a, PARAMETERS> {
    pub fn archetypes_len(&self) -> usize {
        self.borrow.len()
    }
}

pub trait QueryParameterTrait {
    type Result<'a>: for<'b> GetIteratorsTrait<'b>;
    type ResultMut<'a>: for<'b> GetIteratorsTrait<'b>;
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
    type ResultMut<'a> = &'a Vec<A>;

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
    type ResultMut<'a> = &'a mut Vec<A>;

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
    type Result<'a>: for<'b> GetIteratorsTrait<'b>;
    type ResultMut<'a>: for<'b> GetIteratorsTrait<'b>;
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

pub trait GetIteratorsTrait<'a> {
    type Iterator: Iterator;
    type IteratorMut: Iterator;

    fn get_iterator(&'a self) -> Self::Iterator;
    fn get_iterator_mut(&'a mut self) -> Self::IteratorMut;
    fn get_component(&'a self, index: usize) -> <Self::Iterator as Iterator>::Item;
    fn get_component_mut(&'a mut self, index: usize) -> <Self::IteratorMut as Iterator>::Item;
}

impl<'b, T: ComponentTrait> GetIteratorsTrait<'b> for &'_ Vec<T> {
    type Iterator = std::slice::Iter<'b, T>;
    type IteratorMut = std::slice::Iter<'b, T>;

    fn get_iterator(&'b self) -> Self::Iterator {
        self.iter()
    }
    fn get_iterator_mut(&'b mut self) -> Self::IteratorMut {
        self.iter()
    }
    fn get_component(&'b self, index: usize) -> <Self::Iterator as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut(&'b mut self, index: usize) -> <Self::IteratorMut as Iterator>::Item {
        &self[index]
    }
}

impl<'b, T: ComponentTrait> GetIteratorsTrait<'b> for &'_ mut Vec<T> {
    type Iterator = std::slice::Iter<'b, T>;
    type IteratorMut = std::slice::Iter<'b, T>;

    fn get_iterator(&'b self) -> Self::Iterator {
        self.iter()
    }
    fn get_iterator_mut(&'b mut self) -> Self::IteratorMut {
        self.iter()
    }
    fn get_component(&'b self, index: usize) -> <Self::Iterator as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut(&'b mut self, index: usize) -> <Self::IteratorMut as Iterator>::Item {
        &self[index]
    }
}

impl<'b, T: ComponentTrait> GetIteratorsTrait<'b> for RwLockReadGuard<'_, Vec<T>> {
    type Iterator = std::slice::Iter<'b, T>;
    type IteratorMut = std::slice::Iter<'b, T>;

    fn get_iterator(&'b self) -> Self::Iterator {
        self.iter()
    }
    fn get_iterator_mut(&'b mut self) -> Self::IteratorMut {
        self.iter()
    }
    fn get_component(&'b self, index: usize) -> <Self::Iterator as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut(&'b mut self, index: usize) -> <Self::IteratorMut as Iterator>::Item {
        &self[index]
    }
}

impl<'b, T: ComponentTrait> GetIteratorsTrait<'b> for RwLockWriteGuard<'_, Vec<T>> {
    type Iterator = std::slice::Iter<'b, T>;
    type IteratorMut = std::slice::Iter<'b, T>;

    fn get_iterator(&'b self) -> Self::Iterator {
        self.iter()
    }
    fn get_iterator_mut(&'b mut self) -> Self::IteratorMut {
        self.iter()
    }
    fn get_component(&'b self, index: usize) -> <Self::Iterator as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut(&'b mut self, index: usize) -> <Self::IteratorMut as Iterator>::Item {
        &self[index]
    }
}

impl<'b, A: GetIteratorsTrait<'b>, B: GetIteratorsTrait<'b>> GetIteratorsTrait<'b> for (A, B) {
    type Iterator = std::iter::Zip<A::Iterator, B::Iterator>;
    type IteratorMut = std::iter::Zip<A::IteratorMut, B::IteratorMut>;

    fn get_iterator(&'b self) -> Self::Iterator {
        self.0.get_iterator().zip(self.1.get_iterator())
    }
    fn get_iterator_mut(&'b mut self) -> Self::IteratorMut {
        self.0.get_iterator_mut().zip(self.1.get_iterator_mut())
    }
    fn get_component(&'b self, index: usize) -> <Self::Iterator as Iterator>::Item {
        (self.0.get_component(index), self.1.get_component(index))
    }
    fn get_component_mut(&'b mut self, index: usize) -> <Self::IteratorMut as Iterator>::Item {
        (
            self.0.get_component_mut(index),
            self.1.get_component_mut(index),
        )
    }
}

macro_rules! query_iterator_impls {
    // These first two cases are implemented manually so skip them in this macro.
    ($count: tt, ($index0: tt, $tuple0:ident)) => {};
    ($count: tt, ($index0: tt, $tuple0:ident), ($index1: tt, $tuple1:ident)) => {};
    ($count: tt, $( ($index: tt, $tuple:ident) ),* ) => {
        #[allow(unused)]
        impl<'a, $( $tuple: GetIteratorsTrait<'a>,)*> GetIteratorsTrait<'a> for ($( $tuple,)*) {
            type Iterator = MultiIterator<($( $tuple::Iterator,)*)>;
            type IteratorMut = MultiIterator<($( $tuple::IteratorMut,)*)>;
            fn get_iterator(&'a self) -> Self::Iterator {
                MultiIterator::<($( $tuple::Iterator,)*)>::new(($( self.$index.get_iterator(),)*))
            }
            fn get_iterator_mut(&'a mut self) -> Self::IteratorMut {
                MultiIterator::<($( $tuple::IteratorMut,)*)>::new(($( self.$index.get_iterator_mut(),)*))
            }
            #[allow(clippy::unused_unit)]
            fn get_component(&'a self, index: usize) -> <Self::Iterator as Iterator>::Item {
                ($( self.$index.get_component(index),)*)
            }
            #[allow(clippy::unused_unit)]
            fn get_component_mut(&'a mut self, index: usize) -> <Self::IteratorMut as Iterator>::Item {
                ($( self.$index.get_component_mut(index),)*)
            }
        }
    };
}
