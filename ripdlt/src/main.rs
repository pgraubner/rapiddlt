
use rapiddlt::{dlt_v1::{dltit, DltMessageType, DltStorageEntry}, dltbuffer::DltBuffer, DltGrepIterator};
use matchit::{fromgenerator::FromAdaptFnCall, generator::generator::Generator, FromBytesReadableTrait };
use matchit::generator::adapter::AdapterTrait;

const MIN_TIME_DMS: u32 = 18000000; // Timestamp since system start in 0.1 milliseconds

enum ProcessingType {
    Iterative,
    Count,
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
            ProcessingType::Count => count(slice),
            ProcessingType::CtrlMsg => count_hello_world(slice),
            ProcessingType::CtrlMsgRaw => count_hello_world_raw(slice),
            ProcessingType::CtrlMsgGrepIt => count_hello_world_grepit(slice),
            }
        });

    result.sum()
}

fn par_continuous_timestamp_histogram(mmap: DltBuffer) -> BTreeMap<usize, usize>
{
    use rayon::prelude::*;

    use std::thread::available_parallelism;

    let num: usize = available_parallelism().unwrap().get();
    let slices = mmap.partition::<DltStorageEntry>(num);
    println!("available parallelism = {}, slices = {}", num, slices.len());

    let result =
    slices.into_par_iter()
        .map(|slice| {
            continuous_timestamp_histogram(slice)
            }
        );

    result.reduce(|| BTreeMap::new(), |mut acc,next| {
        for (k,v) in next {
            let e = acc.entry(k).or_insert(0);
            *e += v; 
        }
        acc
    })
}

fn par_timestamp_splitit(mmap: DltBuffer) -> BTreeMap<[u8; 4], (BTreeMap<u32, usize>, usize)>
{
    use rayon::prelude::*;

    use std::thread::available_parallelism;

    let num: usize = available_parallelism().unwrap().get();
    let slices = mmap.partition::<DltStorageEntry>(num);
    println!("available parallelism = {}, slices = {}", num, slices.len());

    let result =
    slices.into_par_iter()
        .map(|slice| {
            timestamp_splitit(slice)
            }
        );

    result.reduce(|| BTreeMap::new(), |mut acc,next| {
        for (k,v) in next {
            let mut e = acc.entry(k).or_default();
            for (k1,v1) in v.0 {
                let e1 = (*e).0.entry(k1).or_default();
                *e1 += v1;
            }
            (*e).1 += v.1;
        }
        acc
    })
}


fn count(mmap: &[u8]) -> usize {
    let it = dltit(mmap);

    let result = it.count();
    result
}

fn count_hello_world(mmap: &[u8]) -> usize {
    let finder = memmem::Finder::new("Hello World".as_bytes());

    let it = dltit(mmap);

    let predicate = |a: &DltStorageEntry<'_>| finder.find_iter(a.dlt.payload().expect("needs to be a valid DLT")).next().is_some();
    let result = it
        .filter(predicate)
        .count();
    result
}

fn count_hello_world_raw(mmap: &[u8]) -> usize {
    let finder = memmem::Finder::new("Hello World".as_bytes());
    finder.find_iter(mmap).count()
}

fn count_hello_world_grepit(mmap: &[u8]) -> usize {
    let it = DltGrepIterator::new("H.* World", mmap, 0);
    it.count()
}

// fn lifecycle_histogram_sorted(mmap: DltBuffer) -> BTreeMap<usize, usize> {
//     let it = dltit(mmap.as_slice());
//     let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| b.dlt.timestamp() >= a.dlt.timestamp();

//     let mut result = it
//         .filter(|e| e.dlt.header.header_type.is_with_timestamp())
//         .groupby(predicate)
//         .collect::<Vec<_>>();
//     result
//         .sort_by(|r0, r1| r0.0.dlt.timestamp().unwrap().partial_cmp(&r1.0.dlt.timestamp().unwrap()).unwrap() );
//     result.iter().by_ref()
//         .map(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap())
//         .split(|id| *id as usize / 10000, |_| Generator::count() )
// }

fn continuous_timestamp_histogram(mmap: &[u8]) -> BTreeMap<usize, usize> {
    let it = dltit(mmap);
    let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| b.storage_header.secs.get() >= a.storage_header.secs.get();

    let result = it
        .groupby(predicate)
        .map(|r| r.1.storage_header.secs.get() - r.0.storage_header.secs.get())
    ;
    result.split(|id| *id as usize , |_| Generator::count() )
}

fn mstp_info_histogram(mmap: DltBuffer) -> BTreeMap<(bool, Option<DltMessageType>), usize> {
    let it = dltit(mmap.as_slice());

    it
        .filter(|dlt| dlt.dlt.header.header_type.is_extended_header())
        .split(|dlt|  dlt.dlt.extended_header().unwrap().msin.info(), |_| Generator::count() )
}

fn lifecycle_splitit(mmap: DltBuffer) -> BTreeMap<[u8;4],  (BTreeMap<u32, usize>, usize)> {
    let it = dltit(mmap.as_slice());

    let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| b.dlt.timestamp() >= a.dlt.timestamp();
    let mapping = |r: &(DltStorageEntry<'_>,DltStorageEntry<'_>)| (r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap()) / 10000;

    let result = it
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .split(
            |a: &DltStorageEntry<'_>| a.storage_header.ecu,
            |_| Generator::fork(
                Generator::groupby(predicate)
                                        .map(mapping)
                    .split(
                        |k: &u32| *k,
                        |_| Generator::count()
                    ),
                Generator::count()
            )
        );

    result
}

#[inline(always)]
fn merge_overlapping_timestamps<'a>(a: &(DltStorageEntry<'a>, DltStorageEntry<'a>), b: &(DltStorageEntry<'a>, DltStorageEntry<'a>)) -> Option<(DltStorageEntry<'a>, DltStorageEntry<'a>)>
{
    let max = |a:DltStorageEntry<'a>,b:DltStorageEntry<'a>|
     {if a.storage_header.secs.get() >= b.storage_header.secs.get() { return a }; b};

    if b.0.storage_header.secs.get() >= a.0.storage_header.secs.get() && b.0.storage_header.secs.get() <= a.1.storage_header.secs.get() {
        Some((a.0, max(a.1, b.1)))
    } else if a.0.storage_header.secs.get() >= b.0.storage_header.secs.get() && a.0.storage_header.secs.get() <= b.1.storage_header.secs.get() {
        Some((b.0, max(a.1, b.1)))
    } else {
        None
    }
}

fn timestamp_splitit(mmap: &[u8]) -> BTreeMap<[u8; 4], (BTreeMap<u32, usize>, usize)> { // BTreeMap<[u8; 4], (usize, usize)>
    let it: matchit::NoOffsetIterator<matchit::searchable::readfallbackit::ReadFallbackIterator<'_, DltStorageEntry<'_>>, DltStorageEntry<'_>> = dltit(mmap);

    let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| b.storage_header.secs.get() >= a.storage_header.secs.get();
    let time_delta = |r: &(DltStorageEntry<'_>,DltStorageEntry<'_>)| r.1.storage_header.secs.get() - r.0.storage_header.secs.get();

    let keyfn = |r: &(DltStorageEntry<'_>,DltStorageEntry<'_>)| (r.0.storage_header.secs.get(), r.1.storage_header.secs.get());

    let result = it
        .split(
            |a: &DltStorageEntry<'_>| a.storage_header.ecu,
            |_| Generator::fork(
                Generator::groupby(predicate)
                    .merge(merge_overlapping_timestamps, keyfn )
                    .map(time_delta)
                    .split(
                        |k: &u32| *k,
                        |_| Generator::count()
                    ),
                Generator::count()
            )
        );

    result
}

fn lifecycle_histogram(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let it = dltit(mmap.as_slice());
    let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| a.dlt.ecu_id() == b.dlt.ecu_id() && b.dlt.timestamp() >= a.dlt.timestamp();

    let result = it
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .groupby(predicate)
        .map(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap())
    ;
    result.split(|id| *id as usize / 10000, |_| Generator::count() )
}

fn histogram_payload(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let it = dltit(mmap.as_slice());

    let result = it
        .map(|dlt| dlt.dlt.payload().unwrap_or(&[0u8;0]).len())
    ;
    result.split(|id| *id, |_| Generator::count() )
}

fn histogram_message(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let it = dltit(mmap.as_slice());

    let result = it
        .map(|dlt| dlt.len())
    ;
    result.split(|id| *id, |_| Generator::count() )
}


const IDX_BUCKET_SIZE:usize = 100000000;
fn histogram_hello_world(mmap: DltBuffer) -> BTreeMap<usize, usize> {
    let it = DltGrepIterator::new("H.* World", mmap.as_slice(), 0);
    it.map(|(offset, _)| offset)
        .split(|offset: &usize| offset / IDX_BUCKET_SIZE , |_| Generator::count())
}

fn lifecycle_iter(mmap: &[u8]) -> usize {
    let it = dltit(mmap);
    let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| b.dlt.timestamp() >= a.dlt.timestamp();

    let result = it
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .groupby(predicate)
        .filter(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap() >= MIN_TIME_DMS)
    ;
    let r = result.count();
    println!("{:?} lifecycles >= {}s", r, MIN_TIME_DMS / 10000);
    r
}
use itertools::Itertools;
use memchr::memmem;

fn lifecycle_itertools(mmap: &[u8]) -> usize {
    let it = dltit(mmap);
    let predicate = |a: &DltStorageEntry<'_>, b: &DltStorageEntry<'_>| b.dlt.timestamp() >= a.dlt.timestamp();

    let result = it
        .filter(|e| e.dlt.header.header_type.is_with_timestamp())
        .tuple_windows()
        .filter( |(a,b) |(predicate)(a,b))
        .filter(|r| r.1.dlt.timestamp().unwrap() - r.0.dlt.timestamp().unwrap() >= MIN_TIME_DMS)
    ;
    let r = result.count();
    println!("{:?} lifecycles >= {}s", r, MIN_TIME_DMS / 10000);
    r
}

use std::{env, collections::{BTreeMap}};

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
        "split_lifecycles" =>{
            println!("Distribution of lifecycle durations:");
            for (k,v) in lifecycle_splitit(mmap) {
                println!("{} #lifecycles: {:?}", String::from_utf8(k.to_vec()).unwrap(), v);
            }
        },
        "split_timestamp" =>{
            println!("Durations of periods where DLT storage header timestamps are continuous:");
            for (k,v) in timestamp_splitit(mmap.as_slice()) {
                println!("{} #lifecycles: {:?}", String::from_utf8(k.to_vec()).unwrap(), v);
            }
        },
        "par_split_timestamp" =>{
            println!("Durations of periods where DLT storage header timestamps are continuous:");
            for (k,v) in par_timestamp_splitit(mmap) {
                println!("{} #lifecycles: {:?}", String::from_utf8(k.to_vec()).unwrap(), v);
            }
        },
        "histogram_timestamp" =>{
            println!("Durations of periods where DLT storage header timestamps are continuous:");
            for (k,v) in continuous_timestamp_histogram(mmap.as_slice()) {
                println!("{:?}-{:?} secs: {:?}", k, k+1, v);
            }
        },
        "par_histogram_timestamp" =>{
            println!("Durations of periods where DLT storage header timestamps are continuous:");
            for (k,v) in par_continuous_timestamp_histogram(mmap) {
                println!("{:?}-{:?} secs: {:?}", k, k+1, v);
            }
        },
        "histogram_message_type" =>{
            println!("(verbose, message type): # dlt messages");
            for (mstp,v) in mstp_info_histogram(mmap) {
                println!("{:?}: {:?}", mstp, v);
            }
        },

        "histogram_payload_size" =>{
            println!("Distribution of payload length:");
            let mut total_size = 0;
            for (k,v) in histogram_payload(mmap) {
                let size = k*v;
                println!("{}b: {}, overall: {} kB", k, v, size / 1024);
                total_size += size;
            }
            println!("Payload in total: {} kB", total_size / 1024);
        },
        "histogram_message_size" =>{
            println!("Distribution of DLT message length:");
            let mut total_size = 0;
            for (k,v) in histogram_message(mmap) {
                let size = k*v;
                println!("{}b: {}, overall: {} kB", k, v, size /1024);
                total_size += size;
            }
            println!("DLT messages in total: {} kB", total_size/1024);
        },

        "count" => {
            let r = count(mmap.as_slice());
            println!("{:?} messages", r);
        }

        // Hello World search
        "histogram_hello_world" =>{
            println!("Distribution of 'Hello World' matches:");
            for (k,v) in histogram_hello_world(mmap) {
                println!("Offset {:?}M-{:?}M: {:?}", k*IDX_BUCKET_SIZE / 1000000, (k+1)*IDX_BUCKET_SIZE / 1000000, v);
            }
        },
        "count_hello_world" => {
            let r = count_hello_world(mmap.as_slice());
            println!("{:?} hello world messages", r);
        }
        "count_hello_world_raw" => {
            let r = count_hello_world_raw(mmap.as_slice());
            println!("{:?} raw hello world matches", r);
        }
        "count_hello_world_grepit" => {
            let r = count_hello_world_grepit(mmap.as_slice());
            println!("{:?} hello world messages", r);
        }
        "par_count" => {
            let r = multithreaded(mmap, ProcessingType::Count);
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