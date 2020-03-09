import { parse, GCode } from "./index";

describe("parse", () => {

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
});