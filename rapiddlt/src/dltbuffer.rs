use std::{fs::File, io::{self, BufReader, Read}};

use memmap::MmapOptions;
use matchit::{searchable::{search_last_marker, search_marker, SearchableMarkerTrait}, ContainedBySearch};

use crate::dlt_v1::{DltStorageEntry};

pub enum DltBuffer {
    Mmap(memmap::Mmap),
    Read(Vec<u8>)
}
impl DltBuffer {
    pub fn mmap_file (filename: &str) -> Result<Self,io::Error> {
        let file = File::open(filename)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(DltBuffer::Mmap(mmap))
    }
    pub fn read_file (filename: &str) -> Result<Self,io::Error> {
        let file = File::open(filename)?;
        let mut rdr = BufReader::new(file);
        let mut buf: Vec<u8> = vec![];
        let _ = rdr.read_to_end(&mut buf);
        Ok(DltBuffer::Read(buf))
    }
    pub fn len(&self) -> usize {
        match self {
            DltBuffer::Mmap(mmap) => mmap.len(),
            DltBuffer::Read(vector) => vector.len(),
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        match self {
            DltBuffer::Mmap(mmap) => mmap,
            DltBuffer::Read(vector) => vector.as_slice(),
        }
    }

    pub fn partition_from<'bytes, T: SearchableMarkerTrait<'bytes>>(bytes: &'bytes [u8], num: usize) -> Vec<&'bytes [u8]> {
        let mut result: Vec<&[u8]> = vec![];

        let size = bytes.len() / num;
        let mut candidate = (0, size);
        let search = ContainedBySearch::<DltStorageEntry>::new();

        loop {
            match search_last_marker::<T>(&bytes[..candidate.1]) {
                Some(idx) => {
                    match search.contained_by(bytes, (idx, idx + T::marker().len())) {
                        Some((new_idx, se)) => {
                            if new_idx <= candidate.0 {
                                candidate.0 = match search_marker::<T>(&bytes[new_idx + se.len()..]) {
                                    Some(idx) => idx,
                                    None => bytes.len()
                                };
                                candidate.1 = candidate.0 + size;
                            } else {
                                candidate.1 = new_idx;
                            }
                        },
                        None => {candidate.1 = idx;},
                    };
                },
                None => {
                    candidate.1 = bytes.len();
                },
            }
            if candidate.0 >= bytes.len() || candidate.1 >= bytes.len() {
                break;
            }
            if candidate.0 == candidate.1 {
                break;
            }

            let new_candidate1 = candidate.1 + size;
            if new_candidate1 >= bytes.len() {
                result.push(&bytes[candidate.0..]);
                break;                
            } else {
                result.push(&bytes[candidate.0..candidate.1]);
                candidate = (candidate.1, new_candidate1);    
            }
        }
        result

    }
    pub fn partition<'bytes, T: SearchableMarkerTrait<'bytes>>(&'bytes self, num: usize) -> Vec<&'bytes [u8]> {
        let bytes = self.as_slice();
        Self::partition_from::<T>(bytes, num)
    }
}


#[cfg(test)]
mod tests {
    use crate::dlt_v1::{dltit, DltStorageEntry};
    use matchit::FromBytesReadableTrait;

    use super::*;

    #[test]
    fn test_mmap() {
        let buf = DltBuffer::mmap_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");
        let slice = buf.as_slice();
        assert_eq!(slice.len(), 4686386400);
    }

    #[test]
    fn test_read() {
        let buf = DltBuffer::read_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");
        let slice = buf.as_slice();
        assert_eq!(slice.len(), 4686386400);
    }

    #[test]
    fn partition_count() {
        let buf = DltBuffer::read_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");
        const NUM: usize = 4;
        let slices = buf.partition::<DltStorageEntry>(NUM);

        let mut i = 0;
        for slice in slices.iter() {
            let res = dltit(slice).count();
            i += res;
        }
        assert_eq!(i, 97982400);
    }

    #[test]
    fn partition_nasty_count() {
        let buf = DltBuffer::read_file("../test_gen/nasty_nasty.dlt").expect("create files with test/test_gen.sh");
        const NUM: usize = 500;
        let slices = buf.partition::<DltStorageEntry>(NUM);

        let mut i = 0;
        for slice in slices.iter() {
            let res = dltit(slice).count();
            i += res;
        }
        assert_eq!(i, 50000);
    }


    #[test]
    fn partition_len() {
        let buf = DltBuffer::read_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");
        let num = 215;
        let slices = buf.partition::<DltStorageEntry>(num);

        let mut i = 0;
        for slice in slices.iter() {
            let dlt = DltStorageEntry::try_read(slice);
            assert!(dlt.is_some(), "expect entry {} to be valid: {:?}", i, slice);
            i+=1;
        }
        assert_eq!(slices.iter().map(|s| s.len()).sum::<usize>(), 4686386400);
    }

}
