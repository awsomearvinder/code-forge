port:= "8000"
run data_dir:
	#!/usr/bin/env bash
	cargo run -- {{ data_dir }}
