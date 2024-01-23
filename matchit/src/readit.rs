/// readit
///
/// zerocopy provides FromBytes / IntoBytes traits to read fixed-sized data types without buffer copies.
/// It does not support data types consisting of variable inner types and dynamic detection of endianess / data aligment.
///
/// For this purpose, the traits in this module allow a user to construct a wrapper type.
/// A wrapper type is a struct that can consist of both fixed-sized inner types read with zerocopy
/// complex types with variable size.
///

use std::{mem, marker::PhantomData};

use memchr::memmem;

use zerocopy::{big_endian::U32, FromBytes};
use zerocopy_derive::{FromBytes, FromZeroes};

use crate::{WithOffset, TSearchable, TIterator, search_marker, search_last_marker};


///
/// ReadIterator
///
/// Reads values of type T from a slice of raw bytes.
/// Returns values in the form
///     (offset, T)
/// where offset is the number of bytes read since the
/// start of the slice.
///
/// Type T needs to implement the TWrapFromBytes trait.
///
#[derive(Debug)]
pub struct ReadIterator<'bytes, T>
{
    offset : usize,
    bytes: &'bytes [u8],
    phantom: PhantomData<T>, // used to declare type parameter T, which is used in the iterator trait implementation
}

impl<'bytes, T> ReadIterator<'bytes, T> {
    pub fn new(bytes: &'bytes [u8], offset: usize) -> Self {
        Self { offset, bytes, phantom: PhantomData }
    }
}

impl<'bytes, T> TIterator<'bytes> for ReadIterator<'bytes, T>
        where T: TSearchable<'bytes> {
    fn new(bytes: &'bytes [u8], offset: usize) -> Self {
        ReadIterator {offset, bytes, phantom: PhantomData,
        }
    }
}

impl<'bytes, T> Iterator for ReadIterator<'bytes, T>
    where T: TSearchable <'bytes>
{
    type Item = WithOffset<T>;
    fn next(&mut self) -> Option<WithOffset<T>> {
        if self.offset + T::marker().len() >= self.bytes.len() {
            return None
        }
        if &self.bytes[self.offset..self.offset+T::marker().len()] == T::marker() {
            // a valid T at offset
            match T::try_read(&self.bytes[self.offset..]) {
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
                            self.offset = self.offset + position;
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
