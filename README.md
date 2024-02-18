
# rapiddlt

``rapiddlt`` is an processing library for automotive Diagnostic Log and Trace (DLT) files in Rust.

It is designed to efficiently process large sets of DLT messages for use cases like custom command line tools for fast inspection of DLT files or pre-processing DLT messages for more complex algorithms.

``rapiddlt`` leverages
* zero-copy data structures (with the help of ``zerocopy`` crate),
* memory-mapped files (with the help of ``memmap`` crate),
* iterators that can be #inlined by the Rust compiler.

In addition, ``rapiddlt`` makes use of some experimental tweaks that can further improve processing performance:
* data-parallelism, i.e., multi-threaded DLT processing on slices of DLT files (with the help of ``rayon`` crate),
* binary search using SIMD vector operations (with the help of  ``memchr``, ``regex`` crate).

The basic concept of these experimental tweaks is to ignore the sequential character of a DLT file, perform random accesses on raw data and parse (i.e., reconstruct) the corresponding DLT message only in case something interesting was found. Messages are reconstructed by searching the offset of the last DLT pattern, which is stored in front of every DLT message.

~~These tweaks take into account the possibility of false positive matches, which might not be acceptable in all use cases. On the other hand, if false positive matches are acceptable then these tweaks can significantly boost your processing performance.~~
These tweaks make use of a recursive search to avoid false positive matches. However, this recursive search comes with a high performance penalty, which is acceptable to enable data-parallelism, but may not be acceptable to check a higher number of candidate matches.

``rapiddlt`` is 'work-in-progress'. Do not yet expect stable interfaces. Currently, only DLTv1 files are supported. In order to be future-proof for DLTv2, the main concepts have been abstracted with the help of traits from the internal ``matchit`` library.

## Features

See [CHANGELOG.md](CHANGELOG.md).

## Testing

Test files in ``test/`` were taken from the [adlt](https://github.com/mbehr1/adlt) project.

```bash
# generate a test set with larger file sizes by concatenating existing files:
> test/test_gen.sh
> du -hs test_gen/*
56M     test_gen/lc_ex007_even_larger.dlt
28M     test_gen/lc_ex007_large.dlt
1.1G    test_gen/1_1gb_concat.dlt
4.4G    test_gen/4_4gb_concat.dlt
872K    test_gen/skipped.dlt

> cargo test --release
```

## ripdlt

``ripdlt`` tool implements a few use cases for DLT processing.

Example usage:
```bash
# For benchmarking ripdlt tool, the release version of the binary should be used
> target/release/ripdlt
usage: target/release/ripdlt <file_access_method> <test_name> <filename.dlt>
```

Some usage examples and benchmarks are documented in [BENCHMARKS.md](BENCHMARKS.md).