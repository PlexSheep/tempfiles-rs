#!/bin/bash
set -e
DBURL=sqlite:/tmp/tempfiles-rs/db.sqlite?mode=rwc

cargo run -p migrations -- -u $DBURL $@
