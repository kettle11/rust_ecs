#[doc(hidden)]
/// A series of iterators of the same type that are traversed in a row.
pub struct ChainedIterator<I: Iterator>
where
    I::Item: Iterator,
{
    current_iterator: Option<I::Item>,
    iterators: I,
}

impl<I: Iterator> ChainedIterator<I>
where
    I::Item: Iterator,
{
    #[doc(hidden)]
    pub fn new(mut iterators: I) -> Self {
        let current_iterator = iterators.next();
        Self {
            current_iterator,
            iterators,
        }
    }
}

impl<I: Iterator> Iterator for ChainedIterator<I>
where
    I::Item: Iterator,
{
    type Item = <I::Item as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // Chain the iterators together.
        // If the end of one iterator is reached go to the next.
        loop {
            if let Some(iter) = &mut self.current_iterator {
                if let v @ Some(_) = iter.next() {
                    return v;
                }
            }
            if let Some(i) = self.iterators.next() {
                self.current_iterator = Some(i);
            } else {
                return None;
            }
        }
    }
}
