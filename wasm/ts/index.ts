import * as wasm from "../pkg/index";

export function* parseLines(text: string, callbacks?: Callbacks): Iterable<Line> {
    throw new Error("Not Implemented");
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