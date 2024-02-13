use std::mem;

use zerocopy::{big_endian::U32};
use zerocopy_derive::{FromBytes, FromZeroes};

use crate::{read_typed, searchable::{SearchableMarkerTrait}, FromBytesReadableTrait};

#[derive(Debug,FromBytes, FromZeroes)]
pub struct TestStruct {
    pub pattern: [u8;4],
    pub val: U32
}

#[derive(Debug, Copy, Clone)]
pub struct WrapTestStruct<'bytes> {
    pub inner: &'bytes TestStruct
}

fn try_read(bytes: & [u8]) -> Option<(usize, WrapTestStruct)> {
    let (read_bytes, val) = read_typed::<TestStruct>(bytes)?;
    Some((read_bytes, WrapTestStruct { inner: val }))
}

impl<'bytes> SearchableMarkerTrait<'bytes> for WrapTestStruct<'bytes> {
    fn marker() -> &'static [u8] {
        &[0x1,0x2,0x3,0x4]
    }

    fn try_read_valid_marker(bytes: &'bytes [u8]) -> Option<(usize, Self)> {
        try_read(bytes)
    }

    fn len(&self) -> usize {
        mem::size_of::<WrapTestStruct>()
    }
}


impl<'bytes> FromBytesReadableTrait<'bytes> for WrapTestStruct<'bytes> {

    fn try_read(bytes: &'bytes [u8]) -> Option<(usize, Self)> {
        if bytes[0..4] != [0x1,0x2,0x3,0x4] {
            return None
        }
        try_read(&bytes[4..])
    }

    fn len(&self) -> usize {
        mem::size_of::<WrapTestStruct>()
    }
}

#[cfg(test)]
mod tests {

    use crate::{searchable::{grepit::GrepIterator, readfallbackit::ReadFallbackIterator, searchit::{RevSearchIterator, SearchIterator}}, TIterator, WithOffset};

    use super::*;

    enum TestKind {
        Forward,
        Backward
    }

    fn test_num_matches<'bytes, I>(kind: TestKind)
            where I: TIterator<'bytes> + Iterator<Item = WithOffset<WrapTestStruct<'bytes>>> {
        const N:usize = 4;
        lazy_static! {
            static ref PATTERN: [u8; 8] = [0x1,0x2,0x3,0x4,0x0,0x0,0x0,0x1];
            static ref BYTES: Vec<u8> = {
                PATTERN.repeat(N)
            };
        }
        let it = match kind {
            TestKind::Forward => I::new(BYTES.as_slice(), 0),
            TestKind::Backward => I::new(BYTES.as_slice(), BYTES.len()),
        };

        let mut i = 0;
        for (offset, val) in it {
            i+=1;
            println!("it={:?}", (offset, val));
            assert_eq!(0, offset % mem::size_of::<TestStruct>());
        }
        assert_eq!(N, i);
    }

    fn test_robust_offset<'bytes, I>(kind: TestKind)
            where I: TIterator<'bytes> + Iterator<Item = WithOffset<WrapTestStruct<'bytes>>> {
        const N:usize = 4;

        lazy_static! {
            static ref PATTERN: [u8; 10] = [34,34,0x1,0x2,0x3,0x4,0x0,0x0,0x0,0x1];
            static ref BYTES: Vec<u8> = {
                PATTERN.repeat(N)
            };
        }
        let it = match kind {
            TestKind::Forward => I::new(BYTES.as_slice(), 0),
            TestKind::Backward => I::new(BYTES.as_slice(), BYTES.len()),
        };

        let mut i = 0;
        for (offset, val) in it {
            i+=1;
            println!("it={:?}", (offset, val));
            match kind {
                TestKind::Forward => assert_eq!(0, (offset-2)*i % PATTERN.len()),
                TestKind::Backward => assert_eq!(0, (offset-2)*i % PATTERN.len()),
            };

        }
        assert_eq!(N, i);
    }

    fn test_skipped_simple<'bytes, I>()
        where I: TIterator<'bytes> + Iterator<Item = WithOffset<WrapTestStruct<'bytes>>> {

        const N:usize = 2;
        lazy_static! {
            static ref PATTERN: [u8; 9] = [0x1,0x2,0x3,0x4,0x0,0x0,0x0,0x1,34];
            static ref BYTES: Vec<u8> = {
                PATTERN.repeat(N)
            };
        }
        let mut it = I::new(BYTES.as_slice(), 0);

        let (offset, val) = it.next().expect("expect valid first entry");
        assert_eq!(0, offset);
        assert_eq!(val.inner.val.get(), 0x1);
        let (offset, val) = it.next().expect("expect valid second entry");
        assert_eq!(9, offset);
        assert_eq!(val.inner.val.get(), 0x1);
    }

    fn test_wrap_values<'bytes, I>()
        where I: TIterator<'bytes> + Iterator<Item = WithOffset<WrapTestStruct<'bytes>>> {

        const N:usize = 2;
        lazy_static! {
            static ref PATTERN: [u8; 16] = [0x1,0x2,0x3,0x22,0x22,0x1,0x2,0x3,0x4,0x0,0x0,0x0,0xFF,0x1,0x2,0x3];
            static ref BYTES: Vec<u8> = {
                PATTERN.repeat(N)
            };
        }
        let mut it = I::new(BYTES.as_slice(), 0);
        assert!(it.by_ref()
                .map(|(_, val) | val.inner.val.get())
                .all(|v| v == 0xFF)
        );
    }

    fn test_grepit1() {
        const N:usize = 4;
        lazy_static! {
            static ref PATTERN: [u8; 24] = [0x1,0x2,0x3,0x4,0x0,0x0,0x0,0xFF, 0x1,0x2,0x3,0x4,0xC,0xA,0xF,0xF, 0x1,0x2,0x3,0x4,0x0,0x0,0x0,0xFF];
            static ref BYTES: Vec<u8> = {
                PATTERN.repeat(N)
            };
        }
        let it = GrepIterator::<WrapTestStruct>::new(r"\x0C\x0A\x0F\x0F", BYTES.as_slice(), 0);

        let mut i = 0;
        for (offset, val) in it {
            println!("it={:?}", (offset, val));
            assert_eq!(24*i+8, offset);
            i+=1;
        }
        assert_eq!(N, i);
    }
    fn test_grepit2() {
        const N:usize = 1;
        lazy_static! {
            static ref PATTERN: [u8; 28] = [0xC,0xA,0xF,0xF,0xC,0xA,0xF,0xF, 0x1,0x2,0x3,0x4,0xC,0xA,0xF,0xF, 0x1,0x2,0x3,0x4,0x0,0x0,0x0,0xFF, 0xC,0xA,0xF,0xF];
            static ref BYTES: Vec<u8> = {
                PATTERN.repeat(N)
            };
        }
        let it = GrepIterator::<WrapTestStruct>::new(r"\x0C\x0A\x0F\x0F", BYTES.as_slice(), 0);

        let mut i = 0;
        for (offset, val) in it {
            i+=1;
            println!("it={:?}", (offset, val));
            assert_eq!(i*8, offset);
        }
        assert_eq!(N, i);
    }


    #[test]
    fn searchit_num_matches() {
        test_num_matches::<SearchIterator<WrapTestStruct>>(TestKind::Forward);
    }

    #[test]
    fn searchit_robust_offset() {
        test_robust_offset::<SearchIterator<WrapTestStruct>>(TestKind::Forward);
    }

    #[test]
    fn searchit_skipped_simple() {
        test_skipped_simple::<SearchIterator<WrapTestStruct>>();
    }

    #[test]
    fn searchit_wrap_values() {
        test_wrap_values::<SearchIterator<WrapTestStruct>>();
    }

    #[test]
    fn revsearchit_num_matches() {
        test_num_matches::<RevSearchIterator<WrapTestStruct>>(TestKind::Backward);
    }

    #[test]
    fn revsearchit_robust_offset() {
        test_robust_offset::<RevSearchIterator<WrapTestStruct>>(TestKind::Backward);
    }

    #[test]
    fn revsearchit_wrap_values() {
        test_wrap_values::<RevSearchIterator<WrapTestStruct>>();
    }

    #[test]
    fn readit_num_matches() {
        test_num_matches::<ReadFallbackIterator<WrapTestStruct>>(TestKind::Forward);
    }

    #[test]
    fn readit_robust_offset() {
        test_robust_offset::<ReadFallbackIterator<WrapTestStruct>>(TestKind::Forward);
    }

    #[test]
    fn readit_skipped_simple() {
        test_skipped_simple::<ReadFallbackIterator<WrapTestStruct>>();
    }

    #[test]
    fn readit_wrap_values() {
        test_wrap_values::<ReadFallbackIterator<WrapTestStruct>>();
    }

    #[test]
    fn grepit_test() {
        test_grepit1();
    }

    #[test]
    fn grepit_test2() {
        test_grepit2();
    }

}
