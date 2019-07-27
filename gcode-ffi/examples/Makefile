CRATE_DIR := $(CURDIR)/..

build: basic_usage

run: build

gcode.h: $(CRATE_DIR)/src/lib.rs $(CRATE_DIR)/cbindgen.toml
	cbindgen $(CRATE_DIR) --output $(CURDIR)/gcode.h

libgcode.so: $(CRATE_DIR)/src/lib.rs
	cargo build --manifest-path $(CRATE_DIR)/Cargo.toml
	cp $(CRATE_DIR)/../target/debug/libgcode_ffi.so libgcode.so

basic_usage: basic_usage.c gcode.h libgcode.so
	$(CC) basic_usage.c -o basic_usage -g -Wall -lgcode -L.

clean:
	$(RM) gcode.h
	$(RM) basic_usage

.PHONY: clean