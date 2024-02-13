use std::marker::PhantomData;

use crate::{FromBytesReadableTrait, TIterator, WithOffset};

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
///
/// #[derive(Debug)]
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
        where T: FromBytesReadableTrait<'bytes> {
    fn new(bytes: &'bytes [u8], offset: usize) -> Self {
        ReadIterator {offset, bytes, phantom: PhantomData,
        }
    }
}

impl<'bytes, T> Iterator for ReadIterator<'bytes, T>
    where T: FromBytesReadableTrait <'bytes>
{
    type Item = WithOffset<T>;
    fn next(&mut self) -> Option<WithOffset<T>> {
        if self.offset  >= self.bytes.len() {
            return None
        }
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
    }
}
