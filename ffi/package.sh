#!/bin/sh

set -e

log() {
    echo "[*]" $@ 1>&2
}

GCODE_FFI_DIR=$(realpath $(dirname "$0"))
PROJECT_DIR="$(dirname $GCODE_FFI_DIR)"
TARGET_DIR="$PROJECT_DIR/target"
RELEASE_DIR="$TARGET_DIR/release"
TEMP="$(mktemp -d --tmpdir gcode-tmp.XXXXXXXX)"
VERSION="$(git describe --abbrev=0 | sed s/v//)"

cleanup() {
  log "Removing $TEMP"
  rm -r $TEMP
}

trap cleanup EXIT

log "Compiling"

cargo build --release --all

log "Copying artifacts to $TEMP"
cp "$RELEASE_DIR/libgcode_ffi.a" "$TEMP/libgcode.a"
cp "$RELEASE_DIR/libgcode_ffi.so" "$TEMP/libgcode.so"

log "Generating gcode.h"
GCODE_H="$TEMP/gcode.h"
cbindgen --output "$GCODE_H" "$GCODE_FFI_DIR"

TARGET_TRIPLE="$(rustup target list | grep '(default)' | awk '{print $1}')"
ARCHIVE="$TARGET_DIR/gcode-$VERSION.$TARGET_TRIPLE.zip"
log "Bundling artifacts in $ARCHIVE"
rm -f "$ARCHIVE"
zip --junk-paths "$ARCHIVE" $TEMP/* "$PROJECT_DIR/README.md" "$PROJECT_DIR/LICENSE"
