use dlt_v1::DltStorageEntry;
use matchit::{readit::ReadIterator, searchit::SearchIterator, grepit::GrepIterator};

pub mod dltbuffer;
pub mod dlt_v1;

type DltIterator<'bytes,T> = ReadIterator<'bytes,T>;

pub type DltGrepIterator<'bytes> = GrepIterator<'bytes, DltStorageEntry<'bytes>>;