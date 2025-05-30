#!/bin/sh
set -ex
cd "$(dirname ${0})"
dropdb --if-exist contrib_regression
createdb contrib_regression
psql -d contrib_regression -c "CREATE EXTENSION vector"
PG_REGRESS=$(dirname "$(pg_config --pgxs)")/../test/regress/pg_regress
${PG_REGRESS} --inputdir=./ --bindir='/usr/local/pgsql/bin' --inputdir=test --use-existing --dbname=contrib_regression bit btree cast copy halfvec hnsw_bit hnsw_halfvec hnsw_sparsevec hnsw_vector ivfflat_bit ivfflat_halfvec ivfflat_vector sparsevec vector_type