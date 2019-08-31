/*
 * Usage: LD_LIBRARY_PATH=. ./basic_usage <filename>
 */

#include "gcode.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *filename;
} Args;

int parse_args(int argc, char **argv, Args *result);
void on_line_start(void *user_data, int line_number, Span span);
void on_gcode(void *user_data, Mnemonic mnemonic, int major_number, int minor_number, const Word *args, int arg_len, Span span);
void on_comment(void *user_data, const char *comment, int len, Span span);

typedef struct {
    int lines;
    int gcodes;
    int comments;
} State;

int main(int argc, char **argv) {
    Args args = {};

    int ret = parse_args(argc, argv, &args);
    if (ret != 0) {
        return ret;
    }

    // open the input file and calculate the length
    FILE *f = fopen(args.filename, "rb");
    fseek(f, 0, SEEK_END);
    long file_size = ftell(f);
    fseek(f, 0, SEEK_SET);

    char *buffer = malloc(file_size);
    if (!buffer) {
        return 1;
    }
    fread(buffer, 1, file_size, f);
    fclose(f);

    // set up our state and vtable
    State state = { };
    VTable vtable = {
        .user_data = &state,
        .on_line_start = on_line_start,
        .on_gcode = on_gcode,
        .on_comment = on_comment,
    };

    parse_gcode(buffer, file_size, vtable);

    printf("Finished parsing %s\n", args.filename);
    printf("  Lines: %d\n", state.lines);
    printf("  Total gcodes: %d\n", state.gcodes);
    printf("  Total comments: %d\n", state.comments);

    free(buffer);

    return 0;
}

int parse_args(int argc, char **argv, Args *result) {
    for (int i = 1; i < argc; i++) {
        // handle the help argument
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            printf("Usage: %s <filename>\n", argv[0]);
            return 1;
        } else if (result->filename == NULL) {
            result->filename = argv[i];
        } else {
            printf("Only one file can be parsed at a time\n");
            return 1;
        }
    }

    // were we given a filename
    if (result->filename == NULL) {
        printf("Usage: %s <filename>\n", argv[0]);
        return 1;
    }

    return 0;
}

void on_line_start(void *user_data, int line_number, Span span) {
    State *state = (State*)user_data;
    state->lines += 1;
}

char mnemonic_letter(Mnemonic mn) {
    switch(mn) {
        case MNEMONIC_GENERAL:
            return 'G';
        case MNEMONIC_TOOL_CHANGE:
            return 'T';
        case MNEMONIC_PROGRAM_NUMBER:
            return 'O';
        case MNEMONIC_MISCELLANEOUS:
            return 'M';
        default:
            return '?';
    }
}

void on_gcode(void *user_data, Mnemonic mnemonic, int major_number, int minor_number, const Word *args, int arg_len, Span span) {
    State *state = (State*)user_data;
    state->gcodes += 1;

    printf("%c%d", mnemonic_letter(mnemonic), major_number);

    if (minor_number > 0) {
        printf(".%d", minor_number);
    }

    for (int i = 0; i < arg_len; i++) {
        const Word *arg = &args[i];
        printf(" %c%g", arg->letter, arg->value);
    }

    printf(" @ line %ld\n", span.line+1);
}

void on_comment(void *user_data, const char *comment, int len, Span span) {
    State *state = (State*)user_data;
    state->comments += 1;

    printf("# %.*s\n", len, comment);
}
