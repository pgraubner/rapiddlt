pub mod grepit;
pub mod readfallbackit;
pub mod searchit;

use memchr::memmem;

use crate::FromBytesReadableTrait;

pub trait SearchableMarkerTrait<'bytes>: FromBytesReadableTrait<'bytes>
        where Self: Sized {

    fn marker() -> &'static[u8];
    fn try_read_valid_marker(bytes: &'bytes [u8]) -> Option<(usize, Self)>;
}


pub fn search_marker<'bytes, T>(bytes: &'bytes[u8]) -> Option<usize>
        where T: SearchableMarkerTrait<'bytes> {
    let finder = memmem::Finder::new(T::marker());
    finder.find(bytes)
}

pub fn search_last_marker<'bytes, T>(bytes: &'bytes[u8]) -> Option<usize>
    where T: SearchableMarkerTrait<'bytes> {

    let finder = memmem::FinderRev::new(T::marker());
    finder.rfind(bytes)
}