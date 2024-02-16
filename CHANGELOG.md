# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [0.2.1] - 2024-02-16

### Added
- test: valid verbose DLT files

### Changed
- matchit::generator::merge: changed from reducer to adapter implementation

### Fixed
- Cargo.toml contained wrong versions

## [0.2.0] - 2024-02-13

### Fixed
- test/test_gen.sh: fixed initial run
- ripdlt: fixed histogram_timestamp
- ripdlt: fixed kB size in histogram_payload_size, histogram_message_size
- rapiddlt::dlt_v1::DltStorageHeader: read and store timestamps in little endian
- inline(always) for all iterator next() calls

### Added
- dd_dlt: a tool to read input form stdin and to write the input as valid DLT payload to stdout
- ripdlt: split_timestamp and split_lifecycles commands
- matchit::generator: a module for using the push-based generator pattern instead of the pull-based iterator pattern
    split: splits input by key with the help of a key function, processes each split individually with reducers created by a reducer function
    fork: processes inputs twice with two individual reducers
    fold: a fold on input data
    filter: same as Iterator::filter
    map: same as Iterator::map
    groupby: matches (reflexive-)transitional closures of a relation
    merge: merges values based on a criteria
    count: counts the number of inputs
    sum: sums input values if input implements the Add trait
- matchit::fromgenerator: adapts generators so that they can be combined with iterators
- test/test_gen.sh: added nasty and 1b / 100b payload size test DLTs

### Changed
- matchit::TSearchable: refactored to matchit::searchable::*

### Removed
- matchit::hashmapit:HashMapIterator. Use split from matchit::generator instead
- matchit::matcher. Use matchit::generator instead
- matchit::groupby matcher. Use matchit::generator::groupby instead

## [0.1.0] - 2024-01-23

### Added
- matchit::readit::ReadIterator, parses raw data sequentially, falls back to binary search for markers in case of a corrupted type.
- matchit::searchit::SearchIterator, binary search for markers on raw data.
- matchit::grepit::GrepIterator, binary search for payload within raw data.
- matchit::hashmapit:HashMapIterator, groups parsed values in a hashmap.
- matchit::matcher, a trait for matchers of any kind.
- matchit::groupby matcher, matches reflexive-transitional closures of a relation.
- matchit::testit test cases.

- rapiddlt::dltbuffer, an abstraction for a raw byte reader.
- rapiddlt::dltbuffer::MMap, reading raw bytes mmaped into the process address space.
- rapiddlt::dltbuffer::Read, reading raw bytes with a buffered reader.
- rapiddlt::dlttypes::DltStorageEntry, used to parse DLTv1 files.

- ripdlt: a test tool to process DLTv1 files with rapiddlt and matchit.

[unreleased]: https://github.com/pgraubner/rapiddlt/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/pgraubner/rapiddlt/releases/tag/v0.2.1
[0.2.0]: https://github.com/pgraubner/rapiddlt/releases/tag/v0.2.0
[0.1.0]: https://github.com/pgraubner/rapiddlt/releases/tag/v0.1.0
