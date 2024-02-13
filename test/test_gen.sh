#!/bin/bash

mkdir -p test_gen
cd test

# 612390 DLT entries
for i in {1..5}
do
    cat lc_ex002.dlt lc_ex003.dlt lc_ex004.dlt lc_ex005.dlt lc_ex006.dlt >> ../test_gen/lc_ex007_large.dlt
done

# 11696 DLT entries, after 16 bytes wrong start
echo "wrong_dlt_start" > ../test_gen/skipped.dlt
cat lc_ex002.dlt >> ../test_gen/skipped.dlt
echo "wrong_dlt_middle" >> ../test_gen/skipped.dlt
cat lc_ex002.dlt >> ../test_gen/skipped.dlt
echo "wrong_dlt_end" >> ../test_gen/skipped.dlt

# 1224780 hello world messages
# 56M     lc_ex007_even_larger.dlt
for i in {1..2}
do
    cat ../test_gen/lc_ex007_large.dlt  >> ../test_gen/lc_ex007_even_larger.dlt
done

cd ../test_gen

# 24495600 DLT entries
for i in {1..10}
do
    cat lc_ex007_large.dlt lc_ex007_large.dlt lc_ex007_large.dlt lc_ex007_large.dlt  >> 1_1gb_concat.dlt
done

# 97982400 DLT entries
cat 1_1gb_concat.dlt 1_1gb_concat.dlt 1_1gb_concat.dlt 1_1gb_concat.dlt  > 4_4gb_concat.dlt

cd ..

# 10 lifecycles with 100000 DLT messages each with random payload size=1b
for i in {0..9}
do
    dd if=/dev/urandom bs=100 count=1000 | target/release/dd_dlt ecu$i app$i 1 >> test_gen/1b_random_ten_lcs.dlt
done

# 10 lifecycles with 100000 DLT messages each with random payload size=100b
for i in {0..9}
do
    dd if=/dev/urandom bs=100 count=100000 | target/release/dd_dlt ecu$i app$i 100 >> test_gen/100b_random_ten_lcs.dlt
done

# 10 lifecycles with 10000 DLT messages each with "Hello World" payload size=11b
for i in {0..9}
do
    seq 10000 | xargs -I{} echo -n "Hello World" | target/release/dd_dlt ecu$i app$i 11 >> test_gen/11b_hello_ten_lcs.dlt
done

# 100000 DLT messages with payload size=45b, embeds DLT messages in payload
cat test_gen/11b_hello_ten_lcs.dlt | target/release/dd_dlt nas1 nas1 45 > test_gen/nasty.dlt

# 50000 DLT messages with payload size=158b, embeds 2 DLT messages in a DLT message in payload
cat test_gen/nasty.dlt | target/release/dd_dlt nas2 nas2 158 > test_gen/nasty_nasty.dlt


du -h test*/*.dlt
