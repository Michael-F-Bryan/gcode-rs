import { parse, GCode } from "./index";

describe("gcode parsing", () => {
    it("can parse G90", () => {
        const src = "G90";
        const expected: GCode[] = [
            {
                mnemonic: "G",
                number: 90,
                arguments: {},
                span: {
                    start: 0,
                    end: 2,
                    line: 0,
                }
            },
        ];

        const got = Array.from(parse(src));

        expect(got).toEqual(expected);
    });

    it("can parse more complex items", () => {
        const src = "G01 (the x-coordinate) X50 Y (comment between Y and number) -10.0";
        const expected: GCode[] = [
            {
                mnemonic: "G",
                number: 1,
                arguments: {
                    X: 50,
                    Y: -10
                },
                span: {
                    start: 0,
                    end: src.length,
                    line: 0,
                }
            },
        ];

        const got = Array.from(parse(src));
        console.log(got);

        expect(got).toEqual(expected);
    });
});