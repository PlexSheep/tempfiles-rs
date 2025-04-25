#!/bin/bash
sea-orm-cli generate entity -v \
	--database-url sqlite:/tmp/tempfiles-rs/db.sqlite?mode=rwc \
	--with-serde both \
	-o src/db/schema $@
