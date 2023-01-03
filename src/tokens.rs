use swc_core::ecma::ast::{Expr, JSXOpeningElement};

pub enum MsgToken {
    String(String),
    Value(Box<Expr>),
    TagOpening(TagOpening),
    TagClosing,
    Icu(Icu),
}

pub struct TagOpening {
    pub self_closing: bool,
    pub el: JSXOpeningElement,
}

pub struct Icu {
    pub value: Box<Expr>,
    // todo: JSWord
    pub icu_method: String,
    pub choices: Vec<IcuChoice>,
}

pub struct IcuChoice {
    pub key: String,
    pub tokens: Vec<MsgToken>,
}


// #[cfg(test)]
// mod tests {
//     use super::{*};
//
//     #[test]
//     fn test_normalize_whitespaces() {
//
//     }
// }