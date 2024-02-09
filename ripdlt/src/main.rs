
use rapiddlt::{dlt_v1::{DltStorageEntry, dltit}, dltbuffer::DltBuffer, DltGrepIterator};
use matchit::{matcher::{TMatcherCall}, groupby::{GroupBy}, hashmapit::HasMapIteratorCall, TSearchable, };

const MIN_TIME_DMS: u32 = 18000000; // Timestamp since system start in 0.1 milliseconds

enum ProcessingType {
    Iterative,
    CtrlMsg,
    CtrlMsgRaw,
    CtrlMsgGrepIt
}

fn multithreaded(mmap: DltBuffer, typ: ProcessingType) -> usize {
    use rayon::prelude::*;

    use std::thread::available_parallelism;

    let num: usize = available_parallelism().unwrap().get();
    let slices = mmap.partition::<DltStorageEntry>(num);
    println!("available parallelism = {}, slices = {}", num, slices.len());

    let result =
    slices.into_par_iter()
        .map(|slice| {
            match typ {
            ProcessingType::Iterative => lifecycle_iter(slice),
            ProcessingType::CtrlMsg => count_hello_world(slice),
            ProcessingType::CtrlMsgRaw => count_hello_world_raw(slice),
            ProcessingType::CtrlMsgGrepIt => count_hello_world_grepit(slice),
            }
        });

    result.sum()
}

fn count(mmap: &[u8]) -> usize {
    let it = dltit(mmap);

    let result = it.count();
    result
}

fn count_hello_world(mmap: &[u8]) -> usize {
    let finder = memmem::Finder::new("Hello world".as_bytes());

    let mut it = dltit(mmap);

    let predicate = |a: &DltStorageEntry<'_>| finder.find_iter(a.dlt.payload().expect("needs to be a valid DLT")).next().is_some();
    let result = it.by_ref()
        .filter(predicate)
        .count();
    result
}

fn count_hello_world_raw(mmap: &[u8]) -> usize {
    let finder = memmem::Finder::new("Hello world".as_bytes());
    finder.find_iter(mmap).count()
}

fn count_hello_world_grepit(mmap: &[u8]) -> usize {
    let it = DltGrepIterator::new("H.* world", mmap, 0);
    it.count()
}

fn lifecycle_histogram_sorted(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let mut it = dltit(&mmap.as_slice());
    let predicate = |a: DltStorageEntry<'_>, b: DltStorageEntry<'_>| b.dlt.timestamp() >= a.dlt.timestamp();

    let mut result = it
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .matches(GroupBy::new(predicate))
        .collect::<Vec<_>>();
    result
        .sort_by(|r0, r1| r0.0.dlt.timestamp().unwrap().partial_cmp(&r1.0.dlt.timestamp().unwrap()).unwrap() );
    result.iter().by_ref()
        .map(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap())
        .histogram(|id| *id as usize / 10000 ).collect()
}

fn continuous_timestamp_histogram(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let mut it = dltit(&mmap.as_slice());
    let predicate = |a: DltStorageEntry<'_>, b: DltStorageEntry<'_>| b.storage_header.secs.get() >= a.storage_header.secs.get();

    let result = it.by_ref()
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .matches(GroupBy::new(predicate))
        .map(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap())
    ;
    result.histogram(|id| *id as usize / 10000 ).collect()
}


fn lifecycle_histogram(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let mut it = dltit(&mmap.as_slice());
    let predicate = |a: DltStorageEntry<'_>, b: DltStorageEntry<'_>| a.dlt.ecu_id() == b.dlt.ecu_id() && b.dlt.timestamp() >= a.dlt.timestamp();

    let result = it.by_ref()
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .matches(GroupBy::new(predicate))
        .map(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap())
    ;
    result.histogram(|id| *id as usize / 10000 ).collect()
}

fn histogram_payload(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let mut it = dltit(&mmap.as_slice());

    let result = it.by_ref()
        .map(|dlt| dlt.dlt.payload().unwrap_or(&[0u8;0]).len())
    ;
    result.histogram(|id| *id as usize ).collect()
}

fn histogram_message(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let mut it = dltit(&mmap.as_slice());

    let result = it.by_ref()
        .map(|dlt| dlt.len())
    ;
    result.histogram(|id| *id as usize ).collect()
}


const IDX_BUCKET_SIZE:usize = 100000000;
fn histogram_hello_world(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let it = DltGrepIterator::new("H.* world", mmap.as_slice(), 0);
    it.map(|(offset, _)| offset)
        .histogram(|offset| offset / IDX_BUCKET_SIZE )
        .collect()
}

fn lifecycle_iter(mmap: &[u8]) -> usize {
    let mut it = dltit(mmap);
    let predicate = |a: DltStorageEntry<'_>, b: DltStorageEntry<'_>| b.dlt.timestamp() >= a.dlt.timestamp();

    let result = it.by_ref()
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .matches(GroupBy::new(predicate))
        .filter(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap() >= MIN_TIME_DMS)
    ;
    let r = result.count();
    println!("{:?} lifecycles >= {}s", r, MIN_TIME_DMS / 10000);
    r
}
use itertools::Itertools;
use memchr::memmem;

fn lifecycle_itertools(mmap: &[u8]) -> usize {
    let mut it = dltit(mmap);
    let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| b.dlt.timestamp() >= a.dlt.timestamp();

    let result = it.by_ref()
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .tuple_windows()
        .filter( |(a,b) |(predicate)(a,b))
        .filter(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap() >= MIN_TIME_DMS)
    ;
    let r = result.count();
    println!("{:?} lifecycles >= {}s", r, MIN_TIME_DMS / 10000);
    r
}

use std::{env, collections::{BTreeMap}, mem};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("usage: {} <file_access_method> <test_name> <filename.dlt>", args[0]);
        println!("  runs ripdlt tests with different implementations");

        return;
    }
    println!("{}", args.join(" "));
    let mmethod = &args[1];
    let fname = &args[3];

    let mmap: DltBuffer = match mmethod.as_str() {
        "mmap" => DltBuffer::mmap_file(fname).expect("file not found"),
        "read" => DltBuffer::read_file(fname).expect("file not found"),
        _ => panic!("wrong paramter")
    };

    match args[2].as_str() {
        "histogram_lifecycles" =>{
            println!("Distribution of lifecycle durations:");
            for (k,v) in lifecycle_histogram(mmap) {
                println!("{:?}-{:?} secs: {:?}", k, k+1, v);
            }
        },
        "histogram_timestamp" =>{
            println!("Durations of periods where DLT storage header timestamps are continuous:");
            for (k,v) in continuous_timestamp_histogram(mmap) {
                println!("{:?}-{:?} secs: {:?}", k, k+1, v);
            }
        },

        "histogram_hello_world" =>{
            println!("Distribution of 'Hello World' matches:");
            for (k,v) in histogram_hello_world(mmap) {
                println!("Offset {:?}M-{:?}M: {:?}", k*IDX_BUCKET_SIZE / 1000000, (k+1)*IDX_BUCKET_SIZE / 1000000, v);
            }
        },
        "histogram_payload_size" =>{
            println!("Distribution of payload length:");
            let mut total_size = 0;
            for (k,v) in histogram_payload(mmap) {
                let size = k*v / 1024;
                println!("{}b: {}, overall: {} kB", k, v, size);
                total_size += size;
            }
            println!("Payload in total: {} kB", total_size);
        },
        "histogram_message_size" =>{
            println!("Distribution of DLT message length:");
            let mut total_size = 0;
            for (k,v) in histogram_message(mmap) {
                let size = k*v / 1024;
                println!("{}b: {}, overall: {} kB", k, v, size);
                total_size += size;
            }
            println!("DLT messages in total: {} kB", total_size);
        },

        "count" => {
            let r = count(mmap.as_slice());
            println!("{:?} messages", r);
        }
        "count_hello_world" => {
            let r = count_hello_world(mmap.as_slice());
            println!("{:?} hello world messages", r);
        }
        "count_hello_world_raw" => {
            let r = count_hello_world_raw(mmap.as_slice());
            println!("{:?} hello world messages", r);
        }
        "count_hello_world_grepit" => {
            let r = count_hello_world_grepit(mmap.as_slice());
            println!("{:?} hello world messages", r);
        }
        "par_count_hello_world" => {
            let r = multithreaded(mmap, ProcessingType::CtrlMsg);
            println!("{:?} hello world messages", r);
        }
        "par_count_hello_world_raw" => {
            let r = multithreaded(mmap, ProcessingType::CtrlMsgRaw);
            println!("{:?} hello world messages", r);
        }
        "par_count_hello_world_grepit" => {
            let r = multithreaded(mmap, ProcessingType::CtrlMsgGrepIt);
            println!("{:?} hello world messages", r);
        }
        "par_iter" => {multithreaded(mmap, ProcessingType::Iterative);},
        "iter" => {lifecycle_iter(mmap.as_slice());},
        "itertools" => {lifecycle_itertools(mmap.as_slice());},
        _ => panic!("wrong parameter")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_iter() {
        let mmap: DltBuffer = DltBuffer::mmap_file("../test_gen/1_1gb_concat.dlt").expect("see test/HowTo.md");
        assert_eq!(199, lifecycle_iter(mmap.as_slice()));
    }

    #[test]
    fn test_substring() {
        let mmap: DltBuffer = DltBuffer::mmap_file("../test/lc_ex003.dlt").expect("see test/HowTo.md");
        let finder = memmem::Finder::new("Hello world".as_bytes());

        let it = dltit(mmap.as_slice());

        let predicate = |a: &DltStorageEntry<'_>| finder.find_iter(a.dlt.payload().expect("needs to be a valid DLT")).next().is_some();
        let result = it
           .filter(predicate)
           .count()
        ;
        assert_eq!(1322, result);
        println!("{:?} hello world messages", result);
    }
}