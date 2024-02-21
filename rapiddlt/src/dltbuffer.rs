use std::{fs::File, io::{self, BufReader, Read}};

use memmap::MmapOptions;
use matchit::{searchable::{SearchableMarkerTrait}, partition_from};
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

    ///
    /// ``partition::<T>``: helper function to apply ``DltBuffer::partition_from::<T>`` on the containing buffer
    /// 
    pub fn partition<'bytes, T: SearchableMarkerTrait<'bytes>>(&'bytes self, num: usize) -> Vec<&'bytes [u8]> {
        let bytes = self.as_slice();
        partition_from::<T>(bytes, num)
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
            assert!(dlt.is_some(), "expect entry {} to be valid {:?}", i, &slice[..4]);
            i+=1;
        }
        assert_eq!(slices.iter().map(|s| s.len()).sum::<usize>(), 4686386400);
    }

}
