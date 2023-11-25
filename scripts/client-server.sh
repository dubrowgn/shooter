#! /bin/bash

root_path="$(cd "$(dirname "$0")/.." && pwd)"

cd "$root_path" \
	&& cargo build \
	|| exit

gnome-terminal -- bash -c 'cargo run; read -p "Press any key to close... " -n1 -s' &
gnome-terminal -- bash -c 'cargo run -- --server localhost:8000; read -p "Press any key to close... " -n1 -s' &

wait
