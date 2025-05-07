#!/bin/bash
set -e
DBURL=sqlite:/tmp/tempfiles-rs/db.sqlite?mode=rwc
echo DBURL: $DBURL

cargo run -p migrations -- -u $DBURL $@
