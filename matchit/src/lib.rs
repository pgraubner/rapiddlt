#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

pub mod searchable;
pub mod generator;
pub mod fromgenerator;
pub mod readit;


use std::{marker::PhantomData, mem};

use memchr::memmem::Finder;
use searchable::SearchableMarkerTrait;
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
    fn max_len() -> usize;
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

pub struct ContainedBySearch<T>(Finder<'static>, PhantomData<T>);

impl<'bytes, T> ContainedBySearch<T> 
where
    T: SearchableMarkerTrait<'bytes>
{
    pub fn new() -> Self {
        Self(Finder::new(T::marker()), PhantomData)
    }

    #[inline(always)]
    pub fn contained_by(&self, bytes: &'bytes [u8], payload: (usize, usize)) -> Option<(usize, T)> 
    {
        if payload.0 >= bytes.len() || payload.1 >= bytes.len() || payload.0 >= payload.1 || payload.1 == 0 {
            return None
        }
        
        // go back max payload size
        let backwards_offset = match payload.0.checked_sub(T::max_len()) {
            Some(val) => val,
            None => 0,
        };
        
        let iter = self.0.find_iter(&bytes[backwards_offset..]);
        
        for candidate_idx in iter.map(|x| x + backwards_offset) {
            if candidate_idx >= payload.0 {
                return None
            }
            if let Some((_, container)) = T::try_read(&bytes[candidate_idx..]) {

                // needs to contain payload.1
                if candidate_idx + container.len() >= payload.1 {
                    // recursively check whether the current container is contained in another container
                    return match self.contained_by(bytes, (candidate_idx, candidate_idx + T::marker().len())) {
                        Some(_) => continue,
                        None => Some((candidate_idx, container)),
                    };
                }
            }
        }
        None
    }
}

///
/// ``partition_from::<T>``: create ``num`` partitions from a raw byte slice
///   in a way that valid Ts do not overlap between partitions.
/// 
pub fn partition_from<'bytes, T: SearchableMarkerTrait<'bytes>>(bytes: &'bytes [u8], num: usize) -> Vec<&'bytes [u8]> {
    let mut result: Vec<&[u8]> = vec![];

    let size = bytes.len() / num;
    let mut candidate = (0, size);

    let search = ContainedBySearch::<T>::new();
    loop {
        // out of bounds checks: canidates may not exceed ``bytes`` slice and
        if candidate.0 > bytes.len() || candidate.1 > bytes.len() {
            break;
        }
        candidate.1 = match search.contained_by(bytes, (candidate.1, candidate.1+1)) {
            Some((container ,_)) => {
                // in ~99 percent of the cases, candidate.1 points to a valid T.
                container
            },
            None => {
                // if no containing T was found, candidate.1 points to invalid raw data,
                // which is a valid split between two partitions. 
                candidate.1
            },
        };
        
        // candidate.1 may not point to a container that is part of another partition
        if candidate.0 >= candidate.1 {
            break;
        }

        // store result and check for the next candidate for a split
        if candidate.1 + size /4 >= bytes.len() {
            result.push(&bytes[candidate.0..]);
            break;                
        } else {
            result.push(&bytes[candidate.0..candidate.1]);
            candidate = (candidate.1, candidate.1 + size);    
        }
    }

    result

}
