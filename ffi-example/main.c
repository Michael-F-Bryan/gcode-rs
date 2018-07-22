#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "gcode.h"

// These *should* normally be part of "gcode.h", but due to a bug in cbindgen
// (eqrion/cbindgen#174) they aren't being exported properly.
#define SIZE_OF_PARSER 64
#define SIZE_OF_GCODE 312

void print_gcode(Gcode* gcode);
void print_mnemonic(Gcode* gcode);
void print_args(Gcode* gcode);

int main() {
    bool success = true;
    const char* src = "G01 X123 Y-20.5 G04 P500\nN20 G1";

    // "allocate" some memory for our parser and the parsed gcode. Of course,
    // normally you can just use malloc()
    char parser_buf[SIZE_OF_PARSER];
    char gcode_buf[SIZE_OF_GCODE];

    Parser* parser = (Parser*)parser_buf;
    Gcode* gcode = (Gcode*)gcode_buf;

    success = parser_new(parser, src, strlen(src));
    if (success) {
        while (parser_next(parser, gcode)) {
            print_gcode(gcode);
        }
    }

    if (success) {
        return 0;
    } else {
        printf("Error!\n");
        return 1;
    }
}


void print_gcode(Gcode* gcode) {
    uint32_t line_number;

    if (gcode_line_number(gcode, &line_number)) {
        printf("N%d ", line_number);
    }

    print_mnemonic(gcode);
    printf("%g", gcode_number(gcode));
    print_args(gcode);

    printf("\n");
}


void print_mnemonic(Gcode* gcode) {
    switch (gcode_mnemonic(gcode)) {
        case PROGRAM_NUMBER:
            printf("O");
            break;
        case TOOL_CHANGE:
            printf("T");
            break;
        case MACHINE_ROUTINE:
            printf("M");
            break;
        case GENERAL:
            printf("G");
            break;
        default:
            printf("?");
            break;
    }
}

void print_args(Gcode* gcode) {
    const Word* args = gcode_args(gcode);

    for(int i = 0; i < gcode_num_args(gcode); i++) {
        Word word = args[i];
        printf(" %c%g", word.letter, word.value);
    }
}
