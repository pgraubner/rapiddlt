use std::marker::PhantomData;

use regex::bytes::Regex;
use crate::{TIterator, WithOffset};

use super::{searchit::RevSearchIterator, SearchableMarkerTrait};

#[derive(Debug)]
pub struct GrepIterator<'bytes, T>
{
    offset : usize,
    bytes: &'bytes [u8],
    phantom: PhantomData<T>, // used to declare type parameter T, which is used in the iterator trait implementation
    finder: Regex<>,
    iter: RevSearchIterator<'bytes, T>
}

impl<'bytes, T>  GrepIterator<'bytes, T>
where
    T: SearchableMarkerTrait<'bytes>
{
    pub fn new(pattern: &str, bytes: &'bytes [u8], offset: usize) -> GrepIterator<'bytes, T> {
        GrepIterator { offset, bytes, phantom: PhantomData,
            finder: Regex::new(pattern).unwrap(),
            iter: RevSearchIterator::<T>::new(bytes, offset)
        }
    }
}

impl<'bytes, T> GrepIterator<'bytes, T>
    where T: SearchableMarkerTrait<'bytes>
{
    #[inline(always)]
    pub fn search(&self, bytes: &'bytes [u8], offset: usize) -> Option<WithOffset<T>> {
        if offset >= bytes.len() {
            return None
        }
        let foundit = self.finder.find_at(bytes, offset)?.range();

        match self.iter.search(bytes, foundit.start) {
            Some((_off, val)) => {
                if _off < foundit.start &&  _off + val.len() < foundit.end {
                    // match not included
                    self.search(bytes, foundit.end)
                } else {
                    Some((_off, val))
                }
            },
            None => {
                if foundit.end >= bytes.len() {
                    return None
                }
                self.search(bytes, foundit.end)
            },
        }

    }
}

impl<'bytes, T> Iterator for GrepIterator<'bytes, T>
    where T: SearchableMarkerTrait<'bytes>
{
    type Item = WithOffset<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let (offset, val) = self.search(self.bytes, self.offset)?;

        // we need to go at the end of a DLT entry in order
        // not to emit the same DLT entry twice
        self.offset = offset + val.len();

        Some((offset, val))
    }
}

#[derive(Debug)]
pub struct RegExMatcherIterator<'bytes, T>
{
    offset : usize,
    bytes: &'bytes [u8],
    phantom: PhantomData<T>, // used to declare type parameter T, which is used in the iterator trait implementation
    finder: Regex<>,
    iter: RevSearchIterator<'bytes, T>
}

impl<'bytes, T>  RegExMatcherIterator<'bytes, T>
where
    T: SearchableMarkerTrait<'bytes>
{
    pub fn new(pattern: &str, bytes: &'bytes [u8], offset: usize) -> RegExMatcherIterator<'bytes, T> {
        RegExMatcherIterator {offset, bytes, phantom: PhantomData,
            finder: Regex::new(pattern).unwrap(),
            iter: RevSearchIterator::<T>::new(bytes, offset)
        }
    }
}
impl<'bytes, T> RegExMatcherIterator<'bytes, T>
where
    T: SearchableMarkerTrait<'bytes>
{
    #[inline(always)]
    pub fn capture(&self, bytes: &'bytes [u8], offset: usize) -> Option<WithOffset<(usize,usize)>> {
        if offset >= bytes.len() {
            return None
        }
        let mut locs = self.finder.capture_locations();
        let captures = self.finder.captures_read_at(&mut locs, bytes, offset)?;

        let foundit = captures.range();

        match self.iter.search(bytes, foundit.start) {
            Some((_off, val)) => {
                if _off < foundit.start &&  _off + val.len() < foundit.end {
                    // match not included
                    self.capture(bytes, foundit.end)
                } else {
                    Some((_off, locs.get(1)?))
                }
            },
            None => {
                if foundit.end >= bytes.len() {
                    return None
                }
                self.capture(bytes, foundit.end)
            },
        }
    }
}

impl<'bytes, T> Iterator for RegExMatcherIterator<'bytes, T>
where
    T: SearchableMarkerTrait<'bytes>
{
    type Item = WithOffset<(usize,usize)>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let (offset, val) = self.capture(self.bytes, self.offset)?;

        // we need to go at the end of a DLT entry in order
        // not to emit the same DLT entry twice
        self.offset = offset + val.1;

        Some((offset, val))
    }
}
