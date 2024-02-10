# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Fixed
- test/test_gen.sh: fixed initial run
- DltStorageHeader: read and store timestamps in little endian
- ripdlt: fixed histogram_timestamp
- ripdlt: fixed kB size in histogram_payload_size, histogram_message_size

### Added
- dd_dlt: a tool to read input form stdin and to write the input as valid DLT payload to stdout

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

[unreleased]: https://github.com/pgraubner/rapiddlt/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pgraubner/rapiddlt/releases/tag/v0.1.0
