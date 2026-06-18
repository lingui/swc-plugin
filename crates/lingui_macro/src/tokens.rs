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
    /// Position of the `value` among `cases` in source order, i.e. how many
    /// cases/offsets precede it. Numeric placeholder indices are allocated in
    /// source order, so the value's index must be assigned at this point to
    /// match the JS macro implementation. For the `plural(value, {...})` call
    /// form the value is always first, so this is `0`.
    pub value_pos: usize,
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
