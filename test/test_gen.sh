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
du -h test*/*.dlt
