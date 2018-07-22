CARGO_PROFILE := debug
CARGO_FLAGS :=
GCODE_SO := libgcode.so
GCODE_RS_DIR := ..
INCLUDES := gcode.h
CFLAGS := -std=c99 -L. -Wall -Wpedantic -Wno-pointer-sign -g
LIBRARIES := -lgcode

ifeq ($(CARGO_PROFILE), release)
	CARGO_FLAGS := $(CARGO_FLAGS) --release
endif


example: main.c gcode.h $(GCODE_SO)
	$(CC) main.c $(CFLAGS) -o example $(LIBRARIES)

cargo_build:
	cd $(GCODE_RS_DIR) && cargo build $(CARGO_FLAGS)

gcode.h: 
	cbindgen --output gcode.h $(GCODE_RS_DIR) 2>/dev/null

$(GCODE_SO): cargo_build
	cp $(GCODE_RS_DIR)/target/$(CARGO_PROFILE)/$(GCODE_SO) .

clean:
	$(RM) $(GCODE_SO)
	$(RM) gcode.h
	$(RM) example

.PHONY: clean cargo_build gcode.h
