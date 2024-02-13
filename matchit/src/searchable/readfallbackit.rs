/// readit
///
/// zerocopy provides FromBytes / IntoBytes traits to read fixed-sized data types without buffer copies.
/// It does not support data types consisting of variable inner types and dynamic detection of endianess / data aligment.
///
/// For this purpose, the traits in this module allow a user to construct a wrapper type.
/// A wrapper type is a struct that can consist of both fixed-sized inner types read with zerocopy
/// complex types with variable size.
///

use std::{marker::PhantomData};

use crate::{TIterator, WithOffset};

use super::{search_marker, SearchableMarkerTrait};



#[derive(Debug)]
pub struct ReadFallbackIterator<'bytes, T>
{
    offset : usize,
    bytes: &'bytes [u8],
    phantom: PhantomData<T>, // used to declare type parameter T, which is used in the iterator trait implementation
}

impl<'bytes, T> ReadFallbackIterator<'bytes, T> {
    pub fn new(bytes: &'bytes [u8], offset: usize) -> Self {
        Self { offset, bytes, phantom: PhantomData }
    }
}

impl<'bytes, T> TIterator<'bytes> for ReadFallbackIterator<'bytes, T>
        where T: SearchableMarkerTrait<'bytes> {
    fn new(bytes: &'bytes [u8], offset: usize) -> Self {
        ReadFallbackIterator {offset, bytes, phantom: PhantomData,
        }
    }
}

impl<'bytes, T> Iterator for ReadFallbackIterator<'bytes, T>
    where T: SearchableMarkerTrait<'bytes>
{
    type Item = WithOffset<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<WithOffset<T>> {
        if self.offset + T::marker().len() >= self.bytes.len() {
            return None
        }
        if &self.bytes[self.offset..self.offset+T::marker().len()] == T::marker() {
            // a valid T at offset
            match T::try_read_valid_marker(&self.bytes[self.offset..]) {
                Some((bytes_read, val)) => {
                    let _off = self.offset;
                    self.offset += bytes_read;
                    Some((_off, val))
                },
                None => {
                    None
                }
            }
        } else {
                // no valid T at offset, searching for a marker
                match search_marker::<T>(&self.bytes[self.offset..]) {
                    Some(position) => {
                        if position == 0 {
                            // a match with offset = 0 indicates a slice starting with a valid marker
                            // without containing a valid result
                            None
                        } else {
                            self.offset += position;
                            self.next()
                        }
                    }
                    None => {
                        None
                    },
                }
        }
    }
}
