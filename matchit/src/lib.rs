#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

pub mod searchable;
pub mod generator;
pub mod fromgenerator;
pub mod readit;


use std::mem;

use zerocopy::{FromBytes};

pub type WithOffset<T> = (usize, T);


/// helper to read a zerocopy type T
#[inline(always)]
pub fn read_typed<T>(bytes: &[u8]) -> Option<(usize, &T)>
    where T : FromBytes
{
    let read_bytes = mem::size_of::<T>();
    if read_bytes <= bytes.len() {
        Some((read_bytes, T::ref_from_prefix(bytes)?))
    } else {
        None
    }
}

pub struct NoOffsetIterator<I,T>
        where I: Iterator<Item=WithOffset<T>> {
    pub iter: I
}

impl<I,T> NoOffsetIterator<I, T>
        where I: Iterator<Item=WithOffset<T>> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I,T> Iterator for NoOffsetIterator<I,T>
        where I: Iterator<Item=WithOffset<T>> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let (_, next) = self.iter.next()?;
        Some(next)
    }
}

pub trait TIterator<'a> {
    fn new(bytes: &'a [u8], offset: usize) -> Self;
}

pub trait FromBytesReadableTrait<'bytes>
where
    Self: Sized
{
    fn try_read(bytes: &'bytes [u8]) -> Option<(usize, Self)>;
    fn len(&self) -> usize;
}


/// helper to read a valid zerocopy type T
#[inline(always)]
pub fn read_valid_offset<T, R>(bytes: &[u8], mut isvalid: R) -> Option<(usize, &T)>
    where T : FromBytes, R: FnMut(&T) -> bool
{
    let (offset, result) = read_typed_offset::<T>(bytes)?;
    if (isvalid)(result) {
        Some((offset, result))
    } else {
        None
    }
}


/// helper to read a zerocopy type T
#[inline(always)]
pub fn read_typed_offset<T>(bytes: &[u8]) -> Option<(usize, &T)>
    where T : FromBytes
{
    let offset = mem::size_of::<T>();
    if offset <= bytes.len() {
        Some((offset, T::ref_from_prefix(bytes)?))
    } else {
        None
    }
}
