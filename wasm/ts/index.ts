import * as wasm from "../pkg/index.js";

const { Parser } = wasm;

export function* parseLines(text: string, callbacks?: Callbacks): Iterable<Line> {
    const parser = new Parser(text, callbacks);

    try {
        while (true) {
            const line = parser.next_line();

            if (line) {
                yield toLine(line);
            } else {
                break;
            }
        }

    } finally {
        parser.free();
    }
}

export function* parse(text: string, callbacks?: Callbacks): Iterator<GCode, void, void> {
    for (const line of parseLines(text, callbacks)) {
        for (const gcode of line.gcodes) {
            yield gcode;
        }
    }
}

export type Line = {
    gcodes: GCode[],
    comments: Comment[],
    span: Span,
};

export type Comment = {
    text: string,
    span: Span,
};

export type GCode = {
    mnemonic: string,
    number: number,
    arguments: Map<string, number>,
    span: Span,
};

export type Span = {
    start: number,
    end: number,
    line: number,
}

export interface Callbacks { }

function toLine(line: wasm.Line): Line {
    try {
        throw new Error();
    } finally {
        line.free();
    }
}