use dlt_v1::DltStorageEntry;
use matchit::searchable::{grepit::GrepIterator, readfallbackit::ReadFallbackIterator};

pub mod dltbuffer;
pub mod dlt_v1;

type DltIterator<'bytes,T> = ReadFallbackIterator<'bytes,T>;

pub type DltGrepIterator<'bytes> = GrepIterator<'bytes, DltStorageEntry<'bytes>>;