#!/usr/bin/env bash

# Usage: ./run.sh example_name [--release] [--wasm2map]

# Ensure an example name is given
if [ $# -lt 1 ]; then
  echo "Usage: $0 example_name [--release] [--wasm2map]"
  exit 1
fi

# Initialize variables
EXAMPLE=""
RELEASE=""
MAP=0

# Parse arguments (in any order)
for arg in "$@"; do
  case $arg in
    --release)
      RELEASE="--release"
      ;;
    --wasm2map)
      MAP=1
      ;;
    *)
      # If not a flag, assume it's the example name
      if [ -z "$EXAMPLE" ]; then
        EXAMPLE="$arg"
      else
        echo "Unknown argument: $arg"
        exit 1
      fi
      ;;
  esac
done

# Check that example was given
if [ -z "$EXAMPLE" ]; then
  echo "Error: No example name provided."
  exit 1
fi

echo "Building example: $EXAMPLE"
echo "Release build: ${RELEASE:+yes}"
echo "Generate map: $([[ $MAP -eq 1 ]] && echo yes || echo no)"

# Build
if [[ -n "$RELEASE" ]]; then
  cargo build --release --package "$EXAMPLE" --target=wasm32-unknown-unknown
  TARGET_DIR="release"
else
  cargo build --package "$EXAMPLE" --target=wasm32-unknown-unknown
  TARGET_DIR="debug"
fi

# Copy
cp "target/wasm32-unknown-unknown/$TARGET_DIR/$EXAMPLE.wasm" "examples/webgl/html/$EXAMPLE.wasm"

echo "Copied wasm to examples/webgl/html/$EXAMPLE.wasm"

# wasm2map
if [[ $MAP -eq 1 ]]; then
  echo "Running wasm2map..."
  cargo wasm2map "examples/webgl/html/$EXAMPLE.wasm" --patch --base-url "http://127.0.0.1:5500/examples/webgl/html"
fi

echo "Done!"
