port:= "8000"
run data_dir:
	#!/usr/bin/env bash
	cargo run -- {{ data_dir }}&
	webserver_pid=$!
	trap "kill $webserver_pid" EXIT
	(cd web-frontend; npm run dev -- --port {{ port }})
