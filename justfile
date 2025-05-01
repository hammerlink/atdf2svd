# This is a justfile comment
default:
    @echo "Hello! Run 'just --list' to see available commands"

generate-test-libs:
    mkdir -p ./tests/data/
    -cargo test
    #!/bin/bash
    find tests/data -name "*.svd" -exec bash -c 'file_name=$(basename "$0" .svd); mkdir -p "tests/data/$file_name"; svd2rust -i "$0" -o "tests/data/$file_name" --target none' {} \;

