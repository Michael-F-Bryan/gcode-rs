use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct Span(gcode::Span);

#[wasm_bindgen]
impl Span {
    #[wasm_bindgen(getter)]
    pub fn start(&self) -> usize { self.0.start }

    #[wasm_bindgen(getter)]
    pub fn end(&self) -> usize { self.0.end }

    #[wasm_bindgen(getter)]
    pub fn line(&self) -> usize { self.0.line }
}

impl From<gcode::Span> for Span {
    fn from(other: gcode::Span) -> Span {
        Span(other)
    }
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct Word(gcode::Word);

#[wasm_bindgen]
impl Word {
    #[wasm_bindgen(getter)]
    pub fn letter(&self) -> char { self.0.letter }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f32 { self.0.value }

    #[wasm_bindgen(getter)]
    pub fn span(&self) -> Span { Span(self.0.span) }
}

impl From<gcode::Word> for Word {
    fn from(other: gcode::Word) -> Word {
        Word(other)
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Line(gcode::Line<'static>);

#[wasm_bindgen]
impl Line {
    pub fn num_gcodes(&self) -> usize { self.0.gcodes().len() }

    pub fn get_gcode(&self, index: usize) -> Option<GCode> {
        self.0.gcodes().get(index).map(|g| GCode(g.clone()))
    }

    pub fn num_comments(&self) -> usize { self.0.comments().len() }

    pub fn get_comment(&self, index: usize) -> Option<Comment> {
        self.0.comments().get(index).map(|c| Comment {
            text: c.value.into(),
            span: Span(c.span),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn span(&self) -> Span {
        self.0.span().into()
    }
}

impl From<gcode::Line<'static>> for Line {
    fn from(other: gcode::Line<'static>) -> Line {
        Line(other)
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct GCode(gcode::GCode);

#[wasm_bindgen]
impl GCode {
    #[wasm_bindgen(getter)]
    pub fn mnemonic(&self) -> char { crate::mnemonic_letter(self.0.mnemonic()) }

    #[wasm_bindgen(getter)]
    pub fn number(&self) -> f32 {
        self.0.major_number() as f32 + (self.0.minor_number() as f32) / 10.0
    }

    #[wasm_bindgen(getter)]
    pub fn span(&self) -> Span {
        self.0.span().into()
    }

    pub fn num_arguments(&self) -> usize {
        self.0.arguments().len()
    }

    pub fn get_argument(&self, index: usize) -> Option<Word> {
        self.0.arguments().get(index).copied().map(|w| Word::from(w))
    }
}

impl From<gcode::GCode> for GCode {
    fn from(other: gcode::GCode) -> GCode {
        GCode(other)
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Comment {
    text: String,
    #[wasm_bindgen(readonly)]
    pub span: Span,
}

#[wasm_bindgen]
impl Comment {
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> JsValue { JsValue::from_str(&self.text) }
}

impl<'a> From<gcode::Comment<'a>> for Comment {
    fn from(other: gcode::Comment<'a>) -> Self {
        Comment {
            text: other.value.to_string(),
            span: Span(other.span),
        }
    }
}