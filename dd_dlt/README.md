
# dd_dlt tool

Example usage, using 1b payload with ecu_id = ecu1 and app_id = app1.
```bash
> dd if=/dev/urandom bs=100 count=10000 | target/release/dd_dlt ecu1 app1 1 > test_gen/test.dlt
10000+0 records in
10000+0 records out
1000000 bytes (1,0 MB, 977 KiB) copied, 0,937376 s, 1,1 MB/s
wrote 1000000 DLT messages with payload size=1b
```