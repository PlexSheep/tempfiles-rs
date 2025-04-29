#!/bin/bash
set -e
DBURL=sqlite:/tmp/tempfiles-rs/db.sqlite?mode=rwc

cargo run -p migrations -- up -u $DBURL
sea-orm-cli generate entity -v \
	--database-url $DBURL \
	--with-serde both \
	-o src/db/schema $@
