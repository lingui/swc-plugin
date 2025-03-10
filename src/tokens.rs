use swc_core::ecma::ast::{Expr, JSXOpeningElement};
use swc_core::ecma::atoms::Atom;

pub enum MsgToken {
    String(String),
    Expression(Box<Expr>),
    TagOpening(TagOpening),
    TagClosing,
    IcuChoice(IcuChoice),
}

pub struct TagOpening {
    pub self_closing: bool,
    pub el: JSXOpeningElement,
}

pub struct IcuChoice {
    pub value: Box<Expr>,
    /// plural | select | selectOrdinal
    pub format: Atom,
    pub cases: Vec<CaseOrOffset>,
}

pub enum CaseOrOffset {
    Case(ChoiceCase),
    Offset(String),
}
pub struct ChoiceCase {
    pub key: Atom,
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
