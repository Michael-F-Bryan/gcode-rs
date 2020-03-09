import * as wasm from "../pkg/index";

export type Line = {
    gcodes: GCode[],
    comments: Comment[],
    span: Span,
};

export type Comment = {
    text: string,
    span: Span,
};

type Arguments = { [key: string]: number };

export type GCode = {
    mnemonic: string,
    number: number,
    arguments: Arguments,
    span: Span,
};

export type Span = {
    start: number,
    end: number,
    line: number,
}

export interface Callbacks {
    unknown_content?(text: string, span: Span): void;

    gcode_buffer_overflowed?(
        mnemonic: string,
        number: number,
        span: Span,
    ): void;

    gcode_argument_buffer_overflowed?(
        mnemonic: string,
        number: number,
        argument: wasm.Word,
    ): void;

    comment_buffer_overflow?(
        comment: string,
        span: Span,
    ): void;

    unexpected_line_number?(
        line_number: number,
        span: Span,
    ): void;

    argument_without_a_command?(
        letter: string,
        value: number,
        span: Span,
    ): void;

    number_without_a_letter?(
        value: string,
        span: Span,
    ): void;

    letter_without_a_number?(
        value: string,
        span: Span,
    ): void;
}

export function* parseLines(text: string, callbacks?: Callbacks): Iterable<Line> {
    const parser = new wasm.Parser(text, callbacks);

    try {
        while (true) {
            const line = parser.next_line();

            if (line) {
                yield translateLine(line);
            } else {
                break;
            }
        }

    } finally {
        parser.free();
    }
}

export function* parse(text: string, callbacks?: Callbacks): Iterable<GCode> {
    for (const line of parseLines(text, callbacks)) {
        for (const gcode of line.gcodes) {
            yield gcode;
        }
    }
}

function translateLine(line: wasm.Line): Line {
    try {
        return {
            comments: getAll(line, (l, i) => l.get_comment(i)).map(translateComment),
            gcodes: getAll(line, (l, i) => l.get_gcode(i)).map(translateGCode),
            span: line.span,
        };
    } finally {
        line.free();
    }
}

function translateGCode(gcode: wasm.GCode): GCode {
    const translated = {
        mnemonic: gcode.mnemonic,
        number: gcode.number,
        arguments: translateArguments(gcode),
        span: translateSpan(gcode.span),
    };

    gcode.free();
    return translated;
}

function translateArguments(gcode: wasm.GCode): Arguments {
    const map: Arguments = {};

    for (const word of getAll(gcode, (g, i) => g.get_argument(i))) {
        try {
            map[word.letter] = word.value;
        } finally {
            word.free();
        }
    }

    return map;
}

function translateComment(gcode: wasm.Comment): Comment {
    const translated = {
        text: gcode.text,
        span: translateSpan(gcode.span),
    };

    gcode.free();
    return translated;
}

function translateSpan(span: wasm.Span): Span {
    const translated = {
        start: span.start,
        end: span.end,
        line: span.line,
    };
    span.free();
    return translated;
}

function getAll<TContainer, TItem>(line: TContainer, getter: (line: TContainer, index: number) => TItem | undefined): TItem[] {
    const items = [];
    let i = 0;

    while (true) {
        const item = getter(line, i);

        if (item) {
            items.push(item);
        } else {
            break;
        }

        i++;
    }

    return items;
}
