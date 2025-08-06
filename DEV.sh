#!/usr/bin/env bash

cargo_clean() {
    find . -name "Cargo.toml" -type f | while read -r cargo_file; do
        dir=$(dirname "$cargo_file")
        echo "Running cargo clean in $dir"
        (cd "$dir" && cargo clean)
    done
}



