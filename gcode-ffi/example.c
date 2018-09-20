#include <stdarg.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "gcode.h"

void print_gcode(Gcode* gcode);
void print_mnemonic(Gcode* gcode);
void print_args(Gcode* gcode);
void die (int line_number, const char * format, ...);

int main(int argc, char **argv) {
    const char* src = "G01 X123 Y-20.5 G04 P500\nN20 G1";

    Parser *parser = parser_new(src, strlen(src));
    if (!parser) {
        die(__LINE__, "Unable to create a parser");
    }

    Gcode *gcode = gcode_new();
    if (!gcode) {
        die(__LINE__, "Unable to allocate our gcode");
    }

    while (parser_next(parser, gcode)) {
        print_gcode(gcode);
    }

    gcode_destroy(gcode);
    parser_destroy(parser);
}


void print_gcode(Gcode* gcode) {
    int line_number;

    if (gcode_line_number(gcode, &line_number)) {
        printf("N%d ", line_number);
    }

    print_mnemonic(gcode);
    printf("%02d", gcode_major_number(gcode));

    int minor = gcode_minor_number(gcode);
    if (minor) {
        printf(".%d", minor);
    }

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

    for(int i = 0; i < gcode_arg_count(gcode); i++) {
        Word word = args[i];
        printf(" %c%g", word.letter, word.value);
    }
}

void die (int line_number, const char * format, ...)
{
    va_list vargs;

    va_start(vargs, format);
    fprintf(stderr, "%d: ", line_number);
    vfprintf(stderr, format, vargs);
    fprintf(stderr, ".\n");
    va_end(vargs);
    exit(1);
}

