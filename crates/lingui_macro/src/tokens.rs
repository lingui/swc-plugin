use swc_core::ecma::ast::{Expr, JSXOpeningElement};
use swc_core::ecma::atoms::Atom;

pub enum MsgToken {
    String(String),
    Arg(MsgArg),
    TagOpening(TagOpening),
    TagClosing,
}

pub struct MsgArg {
    pub name: String,
    pub value: Box<Expr>,
    /// plural | select | selectordinal
    pub format: Option<Atom>,
    pub cases: Option<Vec<CaseOrOffset>>,
}

pub struct TagOpening {
    pub self_closing: bool,
    pub el: JSXOpeningElement,
}

pub enum CaseOrOffset {
    Case(ChoiceCase),
    Offset(String),
}

pub struct ChoiceCase {
    pub key: Atom,
    pub tokens: Vec<MsgToken>,
}
