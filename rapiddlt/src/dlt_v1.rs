
use matchit::searchable::SearchableMarkerTrait;
use matchit::{read_typed_offset, FromBytesReadableTrait, NoOffsetIterator};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes};
use zerocopy::{byteorder::big_endian::*, little_endian};

#[derive(AsBytes,FromBytes,FromZeroes,Debug)]
#[repr(C)]
pub struct DltHTyp {
    htyp: u8
}

#[allow(dead_code)]
impl DltHTyp {
    pub fn new(
        is_extended_header: bool,
        is_msb_first: bool,
        is_with_ecu_id: bool,
        is_with_session_id: bool,
        is_with_timestamp: bool,
        version: u8
    ) -> Self {
        let mut htyp = 0b0000_0000;
        if is_extended_header {
            htyp |= 0b0000_0001;
        }
        if is_msb_first {
            htyp |= 0b0000_0010;
        }
        if is_with_ecu_id {
            htyp |= 0b0000_0100;
        }
        if is_with_session_id {
            htyp |= 0b0000_1000;
        }
        if is_with_timestamp {
            htyp |= 0b0001_0000;
        }
        htyp |= version << 5;

        Self { htyp }
    }

    #[inline(always)]
    fn is_extended_header(&self) -> bool {
        (self.htyp & DltHTypMask::UseExtendedHeader as u8) > 0
    }
    #[inline(always)]
    fn is_msb_first(&self) -> bool {
        (self.htyp & DltHTypMask::MostSignificantByteFirst as u8) > 0
    }
    #[inline(always)]
    fn is_with_ecu_id(&self) -> bool {
        (self.htyp & DltHTypMask::WithEcuId as u8) > 0
    }
    #[inline(always)]
    fn is_with_session_id(&self) -> bool {
        (self.htyp & DltHTypMask::WithSessionId as u8) > 0
    }
    #[inline(always)]pub
    fn is_with_timestamp(&self) -> bool {
        (self.htyp & DltHTypMask::WithTimestamp as u8) > 0
    }
    #[inline(always)]
    fn version(&self) -> u8 {
        (self.htyp & DltHTypMask::Version as u8) >> 5
    }
}

#[allow(dead_code)]
#[derive(AsBytes)]
#[repr(u8)]
enum DltHTypMask {
    UseExtendedHeader  = 0x01, // Use Extended Header, UEH
    MostSignificantByteFirst = 0x02,
    WithEcuId = 0x04,
    WithSessionId = 0x08,
    WithTimestamp = 0x10,
    Version = 0xe0,
}

#[derive(AsBytes,FromBytes,FromZeroes,Debug)]
#[repr(C)]
pub struct DltStandardHeader {
    pub header_type: DltHTyp,
    message_counter: u8,  // The message counter is increased with each sent DLT message
    length: U16, // Length of the complete message, without storage header
}

impl DltStandardHeader {
    pub fn new(header_type: DltHTyp, message_counter: u8, length: u16) -> Self {
        Self {
            header_type, message_counter, length: U16::from(length)
        }
    }

    pub fn length(&self) -> usize {
        self.length.get() as usize
    }
}

#[derive(AsBytes,FromBytes,FromZeroes,Debug)]
#[repr(C)]
pub struct DltStorageHeader {
    pub pattern: [u8;4], // This pattern should be DLT0x01
    pub secs: little_endian::U32,  // Seconds since 1.1.1970
    pub msecs: little_endian::I32, // Microseconds
    pub ecu: [u8;4] // The ECU id is added, if it is not already in the DLT message itself
}

impl DltStorageHeader {
    pub fn new(
        pattern: [u8;4],
        secs: u32,
        msecs: i32,
        ecu: [u8;4]
    ) -> Self {
            DltStorageHeader {
                pattern,
                secs: little_endian::U32::from(secs),
                msecs: little_endian::I32::from(msecs),
                ecu,
            }
        }
}

#[derive(AsBytes,FromBytes,FromZeroes,Debug)]
#[repr(C)]
struct DltStandardHeaderExtra {
    pub ecu: [u8;4],            // < ECU id
    pub seid: U32,              // < Session number
    pub tmsp: U32               // < Timestamp since system start in 0.1 milliseconds
}

#[allow(dead_code)]
enum MessageInfoMask {
    Verbose = 0x1,
    MessageType = 0x7 << 1,
    MessageTypeInfo = 0x15 << 4,
}

#[allow(dead_code)]
pub enum MessageType {
    DltTypeLog = 0x0, // Dlt Log Message
    DltTypeAppTrace = 0x1, // Dlt Trace Message
    DltTypeNwTrace = 0x2, // Dlt Network Message
    DltTypeControl = 0x3, // Dlt Control Messag
}

#[allow(dead_code)]
pub enum DltLogMessageTypeInfo {
    DltLogFatal = 0x1,     // Fatal system error
    DltLogDltError = 0x2,   // Application error
    DltLogWarn = 0x3,        // Correct behavior cannot be ensured
    DltLogInfo = 0x4,         // Message of LogLevel type 'Information'
    DltLogDebug = 0x5,       // Message of LogLevel type 'Debug'
    DltLogVerbose = 0x6,     // Message of LogLevel type 'Verbose'
}

#[allow(dead_code)]
pub enum DltTraceMessageTypeInfo {
    DltTraceVariable = 0x1,        // Value of variable
    DltTraceFunctionIn = 0x2,         // Call of a function
    DltTraceFunctionOut = 0x3,        // Return of a function
    DltTraceState = 0x4,       // State of a State Machine
    DltTraceVfb = 0x5,         // RTE events
}
#[allow(dead_code)]
pub enum DltNetworkMessageTypeInfo {
    DltNwTraceIpc = 0x1,      // Inter-Process-Communication
    DltNwTraceCan = 0x2,      // CAN Communications bus
    DltNwTraceFlexray = 0x3,      // FlexRay Communications bus
    DltNwTraceMost = 0x4,         // Most Communications bus
    DltNwTraceEthernet = 0x5,         // Ethernet Communications bus
    DltNwTraceSomeip = 0x6,       // Inter-SOME/IP Communication
}
#[allow(dead_code)]
pub enum DltControlMessageTypeInfo {
    DltControlRequest = 0x1,       // Request Control Message
    DltControlResponse = 0x2,      // Respond Control Message
}

#[derive(AsBytes,FromBytes,FromZeroes,Debug)]
#[repr(C)]
pub struct DltMsinType {
    msin: u8
}

impl DltMsinType {
    pub fn log_message(verbose: bool) -> Self {
        let mut msin: u8 = 0;
        if verbose {
            msin = 0x1;
        }
        Self { msin}
    }

    pub fn trace_message(verbose: bool) -> Self {
        let mut msin: u8 = 0;
        if verbose {
            msin = 0x1;
        }
        Self { msin }
    }

    pub fn network_message(verbose: bool) -> Self {
        let mut msin: u8 = 0;
        if verbose {
            msin = 0x1;
        }
        Self { msin }
    }

    pub fn control_message(verbose: bool) -> Self {
        let mut msin: u8 = 0;
        if verbose {
            msin = 0x1;
        }
        Self { msin }
    }

}


#[derive(AsBytes,FromBytes,FromZeroes,Debug)]
#[repr(C)]
pub struct DltExtendedHeader {
    msin: DltMsinType,         // < messsage info
    noar: u8,         // < number of arguments
    apid: [u8; 4],    // < application id
    ctid: [u8; 4],    // < context id
}

impl DltExtendedHeader {
    pub fn new(msin: DltMsinType, noar: u8, apid: [u8; 4], ctid: [u8; 4]) -> Self {
        Self { msin, noar, apid, ctid }
    }
}

use zerocopy::FromBytes;
use std::mem;

use crate::DltIterator;

#[derive(Debug, Clone, Copy)]

pub struct DltStorageEntry<'bytes> {
    pub storage_header: &'bytes DltStorageHeader,
    pub dlt: DltEntry<'bytes>
}

#[inline(always)]
fn try_read(bytes: &[u8]) -> Option<(usize, DltStorageEntry)> {
    let (size1, sh) = read_typed_offset::<DltStorageHeader>(bytes)?;
    if (size1) > bytes.len() {
        return None
    }
    let (size2, entry) = DltEntry::try_read(&bytes[size1..])?;
    Some((size1+size2, DltStorageEntry {storage_header: sh, dlt: entry}))
}

impl<'bytes> SearchableMarkerTrait<'bytes> for DltStorageEntry<'bytes> {
    #[inline(always)]
    fn marker() -> &'static[u8] {
        &[b'D',b'L',b'T', 0x1]
    }

    #[inline(always)]
    fn try_read_valid_marker(bytes: &'bytes [u8]) -> Option<(usize, Self)> {
        try_read(bytes)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.dlt.header.length() + mem::size_of::<DltStorageHeader>()
    }
}

impl<'bytes> FromBytesReadableTrait<'bytes> for DltStorageEntry<'bytes> {

    #[inline(always)]
    fn try_read(bytes: &'bytes [u8]) -> Option<(usize, Self)> {
        if bytes.len() < 4 {
            return None
        }
        if bytes[0..4] != [b'D',b'L',b'T', 0x1] {
            return None
        }
        try_read(bytes)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.dlt.header.length() + mem::size_of::<DltStorageHeader>()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DltEntry<'bytes> {
    pub header: &'bytes DltStandardHeader,
    pub tail: &'bytes [u8],
}

impl<'bytes> DltEntry<'bytes> {
    #[inline(always)]
    pub fn ecu_id(&self) -> Option<u32> {
        if self.header.header_type.is_with_ecu_id() {
            Some(U32::ref_from_prefix(&self.tail[..4])?.get())
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn timestamp(&self) -> Option<u32> {
        let mut offset = 0usize;
        if self.header.header_type.is_with_timestamp() {
            if self.header.header_type.is_with_ecu_id() {
                offset += 4;
            }
            if self.header.header_type.is_with_session_id() {
                offset += 4;
            }
            if offset > self.tail.len() {
                return None
            }
            Some(U32::ref_from_prefix(&self.tail[offset..])?.get())
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn payload(&self) -> Option<&[u8]> {
        let mut offset = 0usize;
        if self.header.header_type.is_with_ecu_id() {
            offset += 4;
        }
        if self.header.header_type.is_with_session_id() {
            offset += 4;
        }
        if self.header.header_type.is_with_timestamp() {
            offset += 4;
        }
        if self.header.header_type.is_extended_header() {
            let (off, _) = read_typed_offset::<DltExtendedHeader>(self.tail)?;
            offset += off;
        }
        if offset > self.tail.len() {
            return None
        }
        Some(&self.tail[offset..])
    }

}

impl<'bytes> FromBytesReadableTrait<'bytes> for DltEntry<'bytes> {
    #[inline(always)]
    fn try_read(bytes: &'bytes [u8]) -> Option<(usize, Self)> {
        let (size1, h) = read_typed_offset::<DltStandardHeader>(bytes)?;
        if size1 > bytes.len() {
            return None
        }
        let size2 = h.length.get() as usize;
        if size1 > bytes.len() || size2 > bytes.len() || size1 > size2 {
            return None
        }

        let p = &bytes[size1..size2];
        Some((size2, DltEntry {header: h, tail: p}))
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.header.length()
    }
}


/// helper to create an iterator for the wrapper type T
pub fn dltit_offset(b: &[u8] ) -> DltIterator< DltStorageEntry> {
    DltIterator::new(b, 0)
}

pub fn dltit(b: &[u8] ) -> NoOffsetIterator<DltIterator< DltStorageEntry>, DltStorageEntry> {
    NoOffsetIterator::new(DltIterator::new(b, 0))
}

#[cfg(test)]
mod tests {
    use matchit::searchable::grepit::GrepIterator;
    use zerocopy::AsBytes;

    use crate::dltbuffer::DltBuffer;

    use super::*;

    #[test]
    fn storage_header() {
        let sh = DltStorageHeader::ref_from_prefix(&[68, 76, 84,  1,  1,  0,  0,  0,  0,  0,  0,  0, 01, 99, 17, 49]);
        println!("{:?}", sh);
        assert!(sh.is_some());
    }
    #[test]
    fn dlt_robust_iterator_none() {
        let buf = [68, 76, 84, 1, 226, 26, 74, 101, 79, 4, 1, 0, 69, 48, 48, 49, 68, 76, 84, 1, 226, 26, 74, 101, 79, 4, 1, 0, 69, 48, 48, 49, 68, 76, 84, 1,68, 76, 84, 1,68, 76, 84, 1];
        let mut it = dltit(&buf);

        assert!(it.next().is_none());
    }

    #[test]
    fn dlt_robust_iterator_skipped() {
        let mmap = DltBuffer::mmap_file("../test_gen/skipped.dlt").expect("create files with test/test_gen.sh");

        let mut it =dltit(&mmap.as_slice());

        let i: usize = it.by_ref().count();
        assert_eq!(i, 23392);
    }

    #[test]
    fn dlt_robust_iterator_large() {
        let mmap = DltBuffer::mmap_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");

        let mut it = dltit(&mmap.as_slice());
        let i = it.by_ref().count();
        assert_eq!(i, 97982400);
    }

    #[test]
    fn test_iterator_max() {
        let mmap = DltBuffer::mmap_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");

        let mut it = dltit_offset(&mmap.as_slice());
        let i = it.by_ref()
            .map(|(_,e)| e)
            .fold(0, |max, e| if e.dlt.header.length() > max { e.dlt.header.length() } else { max }  );

        assert_eq!(i, 62);
    }
    #[test]
    fn test_iterator_timestamp() {
        let mmap = DltBuffer::mmap_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");

        let mut it = dltit_offset(&mmap.as_slice());
        let filtered_it = it.by_ref()
            .map(|(_,val)| val)
            .filter(|e| e.dlt.header.header_type.is_with_timestamp());
        let i = filtered_it.fold(
            None,|en, n| {
                match en {
                    None => Some(n),
                    Some(e) => {
                        if n.dlt.timestamp() > e.dlt.timestamp() {
                            Some(n)
                        } else {
                            Some(e)
                        }
                    }
                }

            }
        );

        let result = i.unwrap();
        assert_eq!(result.dlt.timestamp(), Some(22149148));
        println!("found in {} entries: {:?}", 97982400, result);
    }

    #[test]
    fn dltiter_length() {
        let buf: DltBuffer = DltBuffer::mmap_file("../test/lc_ex002.dlt").expect("create files with test/test_gen.sh");

        let it = dltit_offset(buf.as_slice());
        assert!(it.map(|(_,val)| val).all(|e| e.dlt.header.length() == 19 || e.dlt.header.length() == 22));
    }

    #[test]
    fn dltentry_empty() {
        let buf = [0u8;0];
        assert!(DltStorageEntry::try_read(&buf).is_none());
        assert!(DltEntry::try_read(&buf).is_none());
    }

    #[test]
    fn dltentry_zeros() {
        let buf = [0u8;100];
        assert!(DltStorageEntry::try_read(&buf).is_none());
        assert!(DltEntry::try_read(&buf).is_none());
    }

    #[test]
    fn dltstorageentry_oneoff() {
        let buf = [68, 76, 84, 1, 226, 26, 74, 101, 79, 4, 1, 0, 69, 48, 48, 49, 49, 63, 0, 62, 0, 0, 132, 198, 65, 4, 65, 48, 49, 49, 67, 48, 48, 49, 0, 130, 0, 0, 38, 0, 45, 45, 97, 110, 111, 110, 44, 114, 101, 99, 101, 112, 116, 105, 111, 110, 95, 116, 105, 109, 101, 58, 49, 54, 57, 57, 51, 53, 53, 51, 54, 50, 48, 54, 54, 109, 115];
        let h = DltStorageEntry::try_read(&buf);
        assert!(h.is_none());
    }
    #[test]
    fn dltstorageentry_onlystorage() {
        let buf = [68, 76, 84, 1, 226, 26, 74, 101, 79, 4, 1, 0, 69, 48, 48, 49];
        let _sh = DltStorageHeader::ref_from_prefix(&buf).expect("expected Storage Header");
        let h = DltStorageEntry::try_read(&buf);
        assert!(h.is_none());
    }

    #[test]
    fn dltstorageentry_incorrect() {
        let buf = [68, 76, 84, 1, 102, 26, 74, 101, 220, 63, 15, 0, 69, 48, 48, 49, 49, 226, 0, 62, 0, 37, 20, 44, 65, 1, 65, 48, 48, 49, 67, 48, 48, 49, 0, 130, 0, 0, 38, 0, 45, 45, 97, 110, 111, 110, 44, 114, 101, 99, 101, 112, 116, 105, 111, 110, 95, 116, 105, 109, 101, 58, 49, 54, 57, 57, 51, 53, 53, 50, 51, 56, 57, 57, 57, 109, 115, 0];
        let dlt = DltEntry::try_read(&buf);
        assert!(dlt.is_none());
    }

    #[test]
    fn dltstorageentry_correct() {
        // let buf: DltBuffer = DltBuffer::mmap_file("../test_gen/lc_ex006.dlt").unwrap();
        // let off = buf.as_slice();
        // println!("{:?}", &off[..(0x4E)]);

        let buf = [68, 76, 84, 1, 102, 26, 74, 101, 220, 63, 15, 0, 69, 48, 48, 49, 49, 226, 0, 62, 0, 37, 20, 44, 65, 1, 65, 48, 48, 49, 67, 48, 48, 49, 0, 130, 0, 0, 38, 0, 45, 45, 97, 110, 111, 110, 44, 114, 101, 99, 101, 112, 116, 105, 111, 110, 95, 116, 105, 109, 101, 58, 49, 54, 57, 57, 51, 53, 53, 50, 51, 56, 57, 57, 57, 109, 115, 0];
        let (offset, se)  = DltStorageEntry::try_read(&buf).expect("valid DLT entry");
        let dlt = &se.dlt;
        println!("storage: {:?}", se);
        assert_eq!(offset, 78);
        assert_eq!(dlt.header.length(), buf.len() - mem::size_of::<DltStorageHeader>());
        assert!(!dlt.header.header_type.is_with_ecu_id());
        assert!(!dlt.header.header_type.is_with_session_id());
        assert!(dlt.header.header_type.is_with_timestamp());
        assert!(dlt.header.header_type.is_extended_header());
        assert_eq!(dlt.timestamp().expect("timetamp expected"), 2429996);
        let p = dlt.payload().expect("payload expected");

        assert_eq!(buf[0x22..].as_bytes(), p.as_bytes());
        println!("payload: {:?}", p);
    }
    #[test]
    fn dltstorageentry_correct2() {
        let buf = [68, 76, 84, 1, 226, 26, 74, 101, 79, 4, 1, 0, 69, 48, 48, 49, 49, 63, 0, 62, 0, 0, 132, 198, 65, 4, 65, 48, 49, 49, 67, 48, 48, 49, 0, 130, 0, 0, 38, 0, 45, 45, 97, 110, 111, 110, 44, 114, 101, 99, 101, 112, 116, 105, 111, 110, 95, 116, 105, 109, 101, 58, 49, 54, 57, 57, 51, 53, 53, 51, 54, 50, 48, 54, 54, 109, 115, 0];
        let (offset, se) = DltStorageEntry::try_read(&buf).expect("valid DLT entry");
        let dlt = &se.dlt;
        assert_eq!(offset, 78);
        assert_eq!(dlt.header.length(), buf.len() - mem::size_of::<DltStorageHeader>());
        assert!(!dlt.header.header_type.is_with_ecu_id());
        assert!(!dlt.header.header_type.is_with_session_id());
        assert!(dlt.header.header_type.is_with_timestamp());
        assert!(dlt.header.header_type.is_extended_header());
        assert_eq!(dlt.timestamp().expect("timetamp expected"), 33990);
        let p = dlt.payload().expect("payload expected");

        assert_eq!(buf[0x22..].as_bytes(), p.as_bytes());
        println!("payload: {:?}", p);
    }

    use crate::{ dlt_v1::{dltit, DltStorageEntry}};

    #[test]
    fn grepit_large() {
        let mmap = DltBuffer::mmap_file("../test_gen/4_4gb_concat.dlt").expect("create files with test/test_gen.sh");

        let it = GrepIterator::<DltStorageEntry>::new("Hello world", &mmap.as_slice(), 0);
        let i = it.count();
        assert_eq!(1057600, i);
    }

    #[test]
    fn grepit_short() {
        let mmap = DltBuffer::mmap_file("../test/lc_ex003.dlt").expect("create files with test/test_gen.sh");

        let it = GrepIterator::<DltStorageEntry>::new("Counter", &mmap.as_slice(), 0);
        let i = it.count();
        assert_eq!(1323, i);
    }

}
