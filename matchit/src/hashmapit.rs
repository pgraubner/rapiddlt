use std::{collections::{BTreeMap}, marker::PhantomData};

#[derive(Debug)]
pub struct HashMapIterator<T, H, I> {
    iter: I,
    hashmap: BTreeMap<usize, usize>,
    hashfn: H,
    phantom: PhantomData<T>, // used to declare type parameter T, which is used in the iterator trait implementation
}

impl<T, H, I> HashMapIterator<T, H, I>
        where H: Fn(&T) -> usize, I: Iterator<Item=T>
{
    #[inline(always)]
    pub fn new(iter: I, hashfn: H) -> Self {
        HashMapIterator { iter, hashfn, hashmap: BTreeMap::new(), phantom: PhantomData }
    }

    pub fn collect(mut self) -> BTreeMap<usize, usize> {
        for _i in self.by_ref() {

        }
        return self.hashmap
    }
}

impl<T, H, I> Iterator for HashMapIterator<T, H, I>
        where H: Fn(&T) -> usize, I: Iterator<Item=T>
    {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<T> {
        for next in self.iter.by_ref() {
            let k = (self.hashfn)(&next);
            *self.hashmap.entry(k).or_insert(0usize) += 1usize;
            return Some(next);
        }
        None
    }
}

pub trait HasMapIteratorCall: Iterator
        where Self: Sized {

    fn histogram<H>(self, hashfn: H) -> HashMapIterator<Self::Item, H, Self>
        where H: Fn(&Self::Item) -> usize
    {
        HashMapIterator::new(self, hashfn)
    }
}

impl<T> HasMapIteratorCall for T where T: Iterator {}
