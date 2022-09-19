use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::*;

pub type QueryBorrowIter<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::Iter<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait>::Iterator<'b>,
    fn(
        &'b (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait>::Iterator<'b>,
>;

pub type QueryBorrowIterMut<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::IterMut<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait>::IteratorMut<'b>,
    fn(
        &'b mut (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::Result<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::Result<'a> as GetIteratorsTrait>::IteratorMut<'b>,
>;

pub type QueryIter<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::Iter<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait>::Iterator<'b>,
    fn(
        &'b (
            ArchetypeInfo,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait>::Iterator<'b>,
>;

pub type QueryIterMut<'a, 'b, PARAMETERS> = std::iter::FlatMap<
    std::slice::IterMut<
        'b,
        (
            ArchetypeInfo<'a>,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    >,
    <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait>::IteratorMut<'b>,
    fn(
        &'b mut (
            ArchetypeInfo,
            <PARAMETERS as QueryParametersTrait>::ResultMut<'a>,
        ),
    )
        -> <<PARAMETERS as QueryParametersTrait>::ResultMut<'a> as GetIteratorsTrait>::IteratorMut<
        'b,
    >,
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
    pub(crate) borrow: Vec<(ArchetypeInfo<'a>, PARAMETERS::Result<'a>)>,
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

pub trait GetIteratorsTrait {
    type Iterator<'a>: Iterator
    where
        Self: 'a;
    type IteratorMut<'a>: Iterator
    where
        Self: 'a;

    fn get_iterator<'a>(&'a self) -> Self::Iterator<'a>;
    fn get_iterator_mut<'a>(&'a mut self) -> Self::IteratorMut<'a>;
    fn get_component<'a>(&'a self, index: usize) -> <Self::Iterator<'a> as Iterator>::Item;
    fn get_component_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> <Self::IteratorMut<'a> as Iterator>::Item;
}

impl<T: ComponentTrait> GetIteratorsTrait for &'_ [T] {
    type Iterator<'b> = std::slice::Iter<'b, T>  where Self: 'b;
    type IteratorMut<'b> = std::slice::Iter<'b, T>  where Self: 'b;

    fn get_iterator<'b>(&'b self) -> Self::Iterator<'b> {
        self.iter()
    }
    fn get_iterator_mut<'b>(&'b mut self) -> Self::IteratorMut<'b> {
        self.iter()
    }
    fn get_component<'b>(&'b self, index: usize) -> <Self::Iterator<'b> as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut<'b>(
        &'b mut self,
        index: usize,
    ) -> <Self::IteratorMut<'b> as Iterator>::Item {
        &self[index]
    }
}

impl<T: ComponentTrait> GetIteratorsTrait for &'_ mut [T] {
    type Iterator<'b> = std::slice::Iter<'b, T>  where Self: 'b;
    type IteratorMut<'b> = std::slice::IterMut<'b, T>  where Self: 'b;

    fn get_iterator<'b>(&'b self) -> Self::Iterator<'b> {
        self.iter()
    }
    fn get_iterator_mut<'b>(&'b mut self) -> Self::IteratorMut<'b> {
        self.iter_mut()
    }
    fn get_component<'b>(&'b self, index: usize) -> <Self::Iterator<'b> as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut<'b>(
        &'b mut self,
        index: usize,
    ) -> <Self::IteratorMut<'b> as Iterator>::Item {
        &mut self[index]
    }
}

impl<T: ComponentTrait> GetIteratorsTrait for RwLockReadGuard<'_, Vec<T>> {
    type Iterator<'b> = std::slice::Iter<'b, T>  where Self: 'b;
    type IteratorMut<'b> = std::slice::Iter<'b, T>  where Self: 'b;

    fn get_iterator<'b>(&'b self) -> Self::Iterator<'b> {
        self.iter()
    }
    fn get_iterator_mut<'b>(&'b mut self) -> Self::IteratorMut<'b> {
        self.iter()
    }
    fn get_component<'b>(&'b self, index: usize) -> <Self::Iterator<'b> as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut<'b>(
        &'b mut self,
        index: usize,
    ) -> <Self::IteratorMut<'b> as Iterator>::Item {
        &self[index]
    }
}

impl<T: ComponentTrait> GetIteratorsTrait for RwLockWriteGuard<'_, Vec<T>> {
    type Iterator<'b> = std::slice::Iter<'b, T> where Self: 'b;
    type IteratorMut<'b> = std::slice::Iter<'b, T> where Self: 'b;

    fn get_iterator<'b>(&'b self) -> Self::Iterator<'b> {
        self.iter()
    }
    fn get_iterator_mut<'b>(&'b mut self) -> Self::IteratorMut<'b> {
        self.iter()
    }
    fn get_component<'b>(&'b self, index: usize) -> <Self::Iterator<'b> as Iterator>::Item {
        &self[index]
    }
    fn get_component_mut<'b>(
        &'b mut self,
        index: usize,
    ) -> <Self::IteratorMut<'b> as Iterator>::Item {
        &self[index]
    }
}

impl<A: GetIteratorsTrait, B: GetIteratorsTrait> GetIteratorsTrait for (A, B) {
    type Iterator<'b> = std::iter::Zip<A::Iterator<'b>, B::Iterator<'b>> where A: 'b, B: 'b;
    type IteratorMut<'b> = std::iter::Zip<A::IteratorMut<'b>, B::IteratorMut<'b>> where A: 'b, B: 'b;

    fn get_iterator<'b>(&'b self) -> Self::Iterator<'b> {
        self.0.get_iterator().zip(self.1.get_iterator())
    }
    fn get_iterator_mut<'b>(&'b mut self) -> Self::IteratorMut<'b> {
        self.0.get_iterator_mut().zip(self.1.get_iterator_mut())
    }
    fn get_component<'b>(&'b self, index: usize) -> <Self::Iterator<'b> as Iterator>::Item {
        (self.0.get_component(index), self.1.get_component(index))
    }
    fn get_component_mut<'b>(
        &'b mut self,
        index: usize,
    ) -> <Self::IteratorMut<'b> as Iterator>::Item {
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
        impl<$( $tuple: GetIteratorsTrait,)*> GetIteratorsTrait for ($( $tuple,)*) {
            type Iterator<'a> = MultiIterator<($( $tuple::Iterator<'a>,)*)> where Self: 'a;
            type IteratorMut<'a> = MultiIterator<($( $tuple::IteratorMut<'a>,)*)> where Self: 'a;
            fn get_iterator<'a>(&'a self) -> Self::Iterator<'a> {
                MultiIterator::<($( $tuple::Iterator<'a>,)*)>::new(($( self.$index.get_iterator(),)*))
            }
            fn get_iterator_mut<'a>(&'a mut self) -> Self::IteratorMut<'a> {
                MultiIterator::<($( $tuple::IteratorMut<'a>,)*)>::new(($( self.$index.get_iterator_mut(),)*))
            }
            #[allow(clippy::unused_unit)]
            fn get_component<'a>(&'a self, index: usize) -> <Self::Iterator<'a> as Iterator>::Item {
                ($( self.$index.get_component(index),)*)
            }
            #[allow(clippy::unused_unit)]
            fn get_component_mut<'a>(&'a mut self, index: usize) -> <Self::IteratorMut<'a> as Iterator>::Item {
                ($( self.$index.get_component_mut(index),)*)
            }
        }
    };
}
