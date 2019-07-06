/* 
 * To compile and run:
 * 
 * $ cargo install cargo-c
 * $ cd gcode/examples/
 * $ cargo cinstall --project-dir .. --destdir /tmp/ --features std
 * $ gcc basic_usage.c -g -lgcode -o basic_usage -L /tmp/usr/lib -I /tmp/usr/include/gcode
 * $ LD_LIBRARY_PATH=/tmp/usr/lib ./basic_usage some_file.gcode
 */ 

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "gcode.h"

void on_unexpected_eof(void *user_data, const TokenKind *expected, int expected_len);
void on_mangled_input(void *user_data, const char *input, int input_len, Span span);
void on_unexpected_token(void *user_data, TokenKind found, Span span, const TokenKind *expected, int expected_len);
void on_start_block(void *user_data, int line_number, int deleted, Span span);
void on_end_block(void *user_data, int line_number, int deleted, Span span);
void on_gcode(void *user_data, int line_number, Mnemonic mnemonic, int major_number, int minor_number, Span span, const Argument *arguments, int argument_len);
void on_comment(void *user_data, Span span, const char *body, int body_len);

int main(int argc, char **argv) {
    for(int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            printf("Usage: %s <filename>\n", argv[0]);
            return 0;
        }
    }

    if (argc != 2) {
        printf("Usage: %s <filename>\n", argv[0]);
        return 1;
    }

    FILE *input = fopen(argv[1], "r");
    if (input == NULL) {
        perror("Unable to open the input file");
        return 1;
    }

    // we need to know how much memory to allocate
    fseek(input, 0, SEEK_END);
    long size = ftell(input);
    rewind(input);

    char *buffer = malloc(size);
    if (buffer == NULL) {
        return 1;
    }

    int bytes_read = fread(buffer, 1, size, input);
    if (bytes_read < 0) {
        perror("Unable to read the input file");
        free(buffer);
        return 1;
    }

    Callbacks cb = {
        .on_gcode=on_gcode,
        .on_comment=on_comment,
        .on_unexpected_eof=on_unexpected_eof,
        .on_mangled_input=on_mangled_input,
        .on_unexpected_token=on_unexpected_token,
        .on_end_block=on_end_block,
    };

    ParseResult result = parse_gcode(buffer, bytes_read, cb);
    if (result != PARSE_RESULT_SUCCESS) {
        printf("Parsing failed");
    }

    free(buffer);
    return result;
}

void on_unexpected_eof(void *user_data, const TokenKind *expected, int expected_len) {
    printf("Unexpected EOF\n");
}

void on_mangled_input(void *user_data, const char *input, int input_len, Span span) {
    printf("Mangled input on line %ld: %.*s\n", span.source_line, input_len, input);
}

void on_unexpected_token(void *user_data, TokenKind found, Span span, const TokenKind *expected, int expected_len) {}

void on_start_block(void *user_data, int line_number, int deleted, Span span) { }

void on_end_block(void *user_data, int line_number, int deleted, Span span) { 
    printf("\n");
}

void on_gcode(void *user_data, int line_number, Mnemonic mnemonic, int major_number, int minor_number, Span span, const Argument *arguments, int argument_len) {
    switch(mnemonic) {
        case MNEMONIC_GENERAL:
            printf("G");
            break;
        case MNEMONIC_MISCELLANEOUS:
            printf("M");
            break;
        case MNEMONIC_TOOL_CHANGE:
            printf("T");
            break;
        case MNEMONIC_PROGRAM_NUMBER:
            printf("O");
            break;
        default:
            printf("%c", mnemonic);
            break;
    }

    printf("%d.%d", major_number, minor_number);

    for (int i = 0; i < argument_len; i++) {
        Argument arg = arguments[i];
        printf(" %c%g", arg.letter, arg.value);
    }

    printf("\n");
}

void on_comment(void *user_data, Span span, const char *body, int body_len) {
    printf("Comment: %.*s\n", body_len, body);
}
