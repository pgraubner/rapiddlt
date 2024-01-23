use std::{marker::PhantomData, mem};

use memchr::{memmem::{self, FindIter, FindRevIter, Finder, FinderRev}};

pub type WithOffset<T> = (usize, T);

#[derive(Debug)]
pub struct SearchIterator<'bytes, T>
{
    offset : usize,
    bytes: &'bytes [u8],
    phantom: PhantomData<T>, // used to declare type parameter T, which is used in the iterator trait implementation
    finder: memmem::Finder<'static>,
}

impl<'bytes, T> TIterator<'bytes> for SearchIterator<'bytes, T>
        where T: TSearchable<'bytes> {
    fn new(bytes: &'bytes [u8], offset: usize) -> SearchIterator<'bytes, T> {
        SearchIterator {offset, bytes, phantom: PhantomData,
            finder: memmem::Finder::new(T::marker()),
        }
    }
}
impl<'bytes, T> SearchIterator<'bytes, T>
        where T: TSearchable<'bytes> {

    pub fn search(&self, bytes: &'bytes [u8], offset: usize) -> Option<WithOffset<T>> {
        if offset >= bytes.len() {
            return None
        }
        let mut candidate = self.finder.find(&bytes[offset..])?;
        candidate += offset;

        let (_, t) = T::try_read(&bytes[candidate..])?;
        Some((candidate, t))
    }
}



impl<'bytes, T> Iterator for SearchIterator<'bytes, T>
    where T: TSearchable<'bytes>
{
    type Item = WithOffset<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.search(self.bytes, self.offset) {
            Some((offset, t)) => {
                self.offset = offset + t.len();
                Some((offset, t))
            }
            None => {
                self.offset += T::marker().len();
                if self.offset >= self.bytes.len() {
                    return None
                }
                self.next()
            }
        }
    }
}

#[derive(Debug)]
pub struct RevSearchIterator<'bytes, T>
{
    pub offset : usize,
    bytes: &'bytes [u8],
    phantom: PhantomData<T>, // used to declare type parameter T, which is used in the iterator trait implementation
    finder: memmem::FinderRev<'static>,
}

impl<'bytes, T> RevSearchIterator<'bytes, T>
        where T: TSearchable<'bytes> {

    pub fn search(&self, bytes: &'bytes [u8], offset: usize) -> Option<WithOffset<T>> {
        if offset > bytes.len() || offset == usize::MAX {
            return None
        }
        let candidate = self.finder.rfind(&bytes[..offset])?;

        let (_, t) = T::try_read(&bytes[candidate..])?;
        Some((candidate, t))
    }
}

impl<'bytes, T> TIterator<'bytes> for RevSearchIterator<'bytes, T>
        where T: TSearchable<'bytes> {
    fn new(bytes: &'bytes [u8], offset: usize) -> RevSearchIterator<'bytes, T> {
        RevSearchIterator {offset, bytes, phantom: PhantomData,
            finder: memmem::FinderRev::new(T::marker()),
        }
    }
}

impl<'bytes, T> Iterator for RevSearchIterator<'bytes, T>
    where T: TSearchable<'bytes>
{
    type Item = WithOffset<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.search(self.bytes, self.offset) {
            Some((offset, t)) => {
                self.offset = offset;
                Some((self.offset, t))
            }
            None => {
                match self.offset.checked_sub(T::marker().len()) {
                    Some(sub) => self.offset = sub,
                    None => self.offset = usize::MAX,
                };
                if self.offset == usize::MAX {
                    return None
                }
                self.next()
            }
        }
    }
}


use zerocopy::{big_endian::U32, FromBytes};
use zerocopy_derive::{FromBytes, FromZeroes};

use crate::{TIterator, TSearchable};


