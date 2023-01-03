use std::collections::HashSet;
use swc_core::{
    common::{DUMMY_SP},
    ecma::{
        ast::*,
    },
};

use crate::{utils};
use crate::tokens::{Icu, MsgToken};

fn dedup_values(mut v: Vec<ValueWithPlaceholder>) -> Vec<ValueWithPlaceholder> {
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(e.placeholder.clone()));

    v
}

pub struct ValueWithPlaceholder {
    pub placeholder: String,
    pub value: Box<Expr>,
}

impl ValueWithPlaceholder {
    pub fn to_prop(self) -> PropOrSpread {
        let ident = Ident::new(self.placeholder.into(), DUMMY_SP);

        PropOrSpread::Prop(Box::new(
            Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(ident),
                value: self.value,
            })
        ))
    }
}

pub struct MessageBuilderResult {
    pub message: Box<Expr>,
    pub values: Option<Box<Expr>>,
    pub components: Option<Box<Expr>>,
}

pub struct MessageBuilder {
    message: String,

    components_stack: Vec<usize>,
    components: Vec<ValueWithPlaceholder>,

    values: Vec<ValueWithPlaceholder>,
    values_indexed: Vec<ValueWithPlaceholder>,
}

impl MessageBuilder {
    pub fn parse(tokens: Vec<MsgToken>) -> MessageBuilderResult {
        let mut builder = MessageBuilder {
            message: String::new(),
            components_stack: Vec::new(),
            components: Vec::new(),
            values: Vec::new(),
            values_indexed: Vec::new(),
        };

        builder.from_tokens(tokens);
        builder.to_args()
    }

    pub fn to_args(mut self) -> MessageBuilderResult {
        let message = Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: utils::normalize_whitespaces(&self.message).into(),
            raw: None,
        })));

        self.values.append(&mut self.values_indexed);

        let values = if self.values.len() > 0 {
            Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: dedup_values(self.values).into_iter().map(|item| item.to_prop()).collect(),
            })))
        } else { None };

        let components = if self.components.len() > 0 {
            Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: self.components.into_iter().map(|item| item.to_prop()).collect(),
            })))
        } else { None };

        MessageBuilderResult {
            message,
            values,
            components,
        }
    }

    fn from_tokens(&mut self, tokens: Vec<MsgToken>) {
        for token in tokens {
            match token {
                MsgToken::String(str) => {
                    self.push_msg(&str);
                }

                MsgToken::Expression(val) => {
                    let placeholder = self.push_exp(val);
                    self.push_msg(&format!("{{{placeholder}}}"));
                }

                MsgToken::TagOpening(val) => {
                    self.push_tag_opening(val.el, val.self_closing);
                }
                MsgToken::TagClosing => {
                    self.push_tag_closing();
                }
                MsgToken::Icu(icu) => {
                    self.push_icu(icu);
                }
            }
        }
    }

    fn push_msg(&mut self, val: &str) {
        self.message.push_str(val);
    }

    fn push_tag_opening(&mut self, el: JSXOpeningElement, self_closing: bool) {
        let current = self.components.len();
        if self_closing {
            self.push_msg(&format!("<{current}/>"));
        } else {
            self.components_stack.push(current);
            self.push_msg(&format!("<{current}>"));
        }

        // todo: it looks very dirty and bad to cloning this jsx values
        self.components.push(ValueWithPlaceholder {
            placeholder: self.components.len().to_string(),
            value: Box::new(Expr::JSXElement(
                Box::new(JSXElement {
                    opening: el,
                    closing: None,
                    children: vec![],
                    span: DUMMY_SP,
                })
            )),
        });
    }

    fn push_tag_closing(&mut self) {
        if let Some(index) = self.components_stack.pop() {
            self.push_msg(&format!("</{index}>"));
        } else {
            // todo JSX tags mismatch. write tests for tags mismatch, swc should not crash in that case
        }
    }

    fn push_exp(&mut self, exp: Box<Expr>) -> String {
        match exp.as_ref() {
            Expr::Ident(ident) => {
                self.values.push(ValueWithPlaceholder {
                    placeholder: ident.sym.to_string().clone(),
                    value: exp.clone(),
                });

                return ident.sym.to_string();
            }
            _ => {
                let index = self.values_indexed.len().to_string();

                self.values_indexed.push(ValueWithPlaceholder {
                    placeholder: index.clone(),
                    value: exp.clone(),
                });

                return index;
            }
        }
    }

    fn push_icu(&mut self, icu: Icu) {
        let value_placeholder = self.push_exp(icu.value);
        let method = icu.icu_method;
        self.push_msg(&format!("{{{value_placeholder}, {method},"));

        for choice in icu.choices {
            let key = choice.key;

            self.push_msg(&format!(" {key} {{"));
            self.from_tokens(choice.tokens);
            self.push_msg("}");
        }

        self.push_msg("}");
    }
}