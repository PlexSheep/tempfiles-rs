#!/bin/bash
set -e
DBURL=sqlite:/tmp/tempfiles-rs/db.sqlite?mode=rwc

cargo run -p migrations -- fresh -u $DBURL
sea-orm-cli generate entity -v \
	--database-url $DBURL \
	--with-serde both \
	--serde-skip-hidden-column \
	-o src/db/schema $@
