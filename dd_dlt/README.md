
# dd_dlt tool

Example usage,
```bash
> dd if=/dev/urandom bs=1 count=10000 | target/release/dd_dlt ecu1 app1 1 > test_gen/test.dlt
10000+0 records in
10000+0 records out
10000 bytes (10 kB, 9.8 KiB) copied, 0.0547122 s, 183 kB/s
wrote 10000 DLT messages with payload size=1b
```