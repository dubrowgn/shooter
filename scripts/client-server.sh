#! /bin/bash

root_path="$(cd "$(dirname "$0")/.." && pwd)"

cd "$root_path" \
	&& cargo build \
	|| exit

gnome-terminal -- cargo run &
gnome-terminal -- cargo run -- --server localhost:8000 &

wait
