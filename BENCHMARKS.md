# ripdlt benchmarks

All the ``ripdlt`` calls listed below were performed on 'hot' caches, i.e., the file was previously loaded from persistent storage into the page cache. CPU: Core-i5 @3.4 GHz, 32 GB RAM.
The release version of the binary is used.

```bash
> du -hs test_gen/4_4gb_concat.dlt
4.4G    test_gen/4_4gb_concat.dlt
```

Durations of periods where DLT storage header timestamps are continuous:
```bash
> time target/release/ripdlt mmap histogram_timestamp test_gen/4_4gb_concat.dlt
Durations of periods where DLT storage header timestamps are continuous:
2-3 secs: 800
2072-2073 secs: 800
88744928-88744929 secs: 800

real    0m1.317s
user    0m1.147s
sys     0m0.169s
```

Durations of periods where DLT timestamps are monotonic for the same ECU ID:
```bash
> time target/release/ripdlt mmap histogram_lifecycles test_gen/4_4gb_concat.dlt
Distribution of lifecycle durations:
0-1 secs: 1516801
1-2 secs: 172800
2-3 secs: 25600
4-5 secs: 800
5-6 secs: 800
118-119 secs: 800
236-237 secs: 800
271-272 secs: 1
1766-1767 secs: 800
2212-2213 secs: 799

real    0m2.061s
user    0m1.851s
sys     0m0.210s
```

Distribution of the size of the DLT payload:
```bash
> time target/release/ripdlt mmap histogram_payload_size test_gen/4_4gb_concat.dlt
Distribution of payload length:
4b: 14032800, overall: 54815 kB
5b: 5600, overall: 27 kB
7b: 85600, overall: 585 kB
10b: 9600, overall: 93 kB
11b: 800, overall: 8 kB
13b: 69883200, overall: 887189 kB
15b: 2116000, overall: 30996 kB
17b: 40000, overall: 664 kB
18b: 2116800, overall: 37209 kB
19b: 1057600, overall: 19623 kB
20b: 1058400, overall: 20671 kB
39b: 2400, overall: 91 kB
44b: 7573600, overall: 325428 kB
Payload in total: 1377399 kB

real    0m1.417s
user    0m1.216s
sys     0m0.201s
```
Distribution of the size of DLT messages:
```bash
> time target/release/ripdlt mmap histogram_message_size test_gen/4_4gb_concat.dlt
Distribution of DLT message length:
28b: 2972000, overall: 81265 kB
35b: 3200, overall: 109 kB
37b: 85600, overall: 3092 kB
38b: 11060800, overall: 410459 kB
39b: 2400, overall: 91 kB
41b: 800, overall: 32 kB
44b: 9600, overall: 412 kB
45b: 2116000, overall: 92988 kB
47b: 69884000, overall: 3207566 kB
48b: 2116800, overall: 99225 kB
49b: 1057600, overall: 50607 kB
50b: 1058400, overall: 51679 kB
51b: 39200, overall: 1952 kB
73b: 2400, overall: 171 kB
78b: 7573600, overall: 576895 kB
DLT messages in total: 4576543 kB

real    0m1.973s
user    0m1.753s
sys     0m0.221s
```
Searching for the regular expression `H.* world` within the payload of each DLT meassage:
```bash
> time target/release/ripdlt mmap count_hello_world test_gen/4_4gb_concat.dlt
1057600 hello world messages

real    0m2.357s
user    0m2.115s
sys     0m0.242s
```
Experimental tweak: Searching for the regular expression `H.* world` within raw data, reconstrucing the DLT message for each match:
```bash
> time target/release/ripdlt mmap count_hello_world_grepit test_gen/4_4gb_concat.dlt
1057600 hello world messages

real    0m0.834s
user    0m0.674s
sys     0m0.160s
```

Experimental tweak: Searching for the regular expression `H.* world` within raw data on 4 CPU cores in parallel, reconstrucing the DLT message for each match:
```bash
> time target/release/ripdlt mmap par_count_hello_world_grepit test_gen/4_4gb_concat.dlt
available parallelism = 4, slices = 4
1057600 hello world messages

real    0m0.324s
user    0m0.836s
sys     0m0.216s
```
