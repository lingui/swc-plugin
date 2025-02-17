use crate::ast_utils::expand_ts_as_expr;
use crate::tokens::{CaseOrOffset, IcuChoice, MsgToken};
use std::collections::HashSet;
use swc_core::{
    common::{SyntaxContext, DUMMY_SP},
    ecma::ast::*,
};

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
        let ident = IdentName::new(self.placeholder.into(), DUMMY_SP);

        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(ident),
            value: self.value,
        })))
    }
}

pub struct MessageBuilderResult {
    pub message_str: String,
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
        let message_str = self.message;

        let message = Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: message_str.clone().into(),
            raw: None,
        })));

        self.values.append(&mut self.values_indexed);

        let values = if self.values.len() > 0 {
            Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: dedup_values(self.values)
                    .into_iter()
                    .map(|item| item.to_prop())
                    .collect(),
            })))
        } else {
            None
        };

        let components = if self.components.len() > 0 {
            Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: self
                    .components
                    .into_iter()
                    .map(|item| item.to_prop())
                    .collect(),
            })))
        } else {
            None
        };

        MessageBuilderResult {
            message_str: message_str.to_string(),
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
                MsgToken::IcuChoice(icu) => {
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
            value: Box::new(Expr::JSXElement(Box::new(JSXElement {
                opening: el,
                closing: None,
                children: vec![],
                span: DUMMY_SP,
            }))),
        });
    }

    fn push_tag_closing(&mut self) {
        if let Some(index) = self.components_stack.pop() {
            self.push_msg(&format!("</{index}>"));
        } else {
            // todo JSX tags mismatch. write tests for tags mismatch, swc should not crash in that case
        }
    }

    fn push_exp(&mut self, mut exp: Box<Expr>) -> String {
        exp = expand_ts_as_expr(exp);

        match exp.as_ref() {
            Expr::Ident(ident) => {
                self.values.push(ValueWithPlaceholder {
                    placeholder: ident.sym.to_string().clone(),
                    value: exp.clone(),
                });

                ident.sym.to_string()
            }
            Expr::Object(object) => {
                if let Some(PropOrSpread::Prop(prop)) = object.props.first() {
                    // {foo}
                    if let Some(short) = prop.as_shorthand() {
                        self.values_indexed.push(ValueWithPlaceholder {
                            placeholder: short.sym.to_string(),
                            value: Box::new(Expr::Ident(Ident {
                                span: DUMMY_SP,
                                sym: short.sym.clone(),
                                ctxt: SyntaxContext::empty(),
                                optional: false,
                            })),
                        });

                        return short.sym.to_string();
                    }
                    // {foo: bar}
                    if let Prop::KeyValue(kv) = prop.as_ref() {
                        if let PropName::Ident(ident) = &kv.key {
                            self.values_indexed.push(ValueWithPlaceholder {
                                placeholder: ident.sym.to_string(),
                                value: kv.value.clone(),
                            });

                            return ident.sym.to_string();
                        }
                    }
                }

                // fallback for {...spread} or {}
                let index = self.values_indexed.len().to_string();

                self.values_indexed.push(ValueWithPlaceholder {
                    placeholder: index.clone(),
                    value: exp.clone(),
                });

                index
            }
            _ => {
                let index = self.values_indexed.len().to_string();

                self.values_indexed.push(ValueWithPlaceholder {
                    placeholder: index.clone(),
                    value: exp.clone(),
                });

                index
            }
        }
    }

    fn push_icu(&mut self, icu: IcuChoice) {
        let value_placeholder = self.push_exp(icu.value);
        let method = icu.format;
        self.push_msg(&format!("{{{value_placeholder}, {method},"));

        for choice in icu.cases {
            match choice {
                // produce offset:{number}
                CaseOrOffset::Offset(val) => {
                    self.push_msg(&format!(" offset:{val}"));
                }
                CaseOrOffset::Case(choice) => {
                    let key = choice.key;

                    self.push_msg(&format!(" {key} {{"));
                    self.from_tokens(choice.tokens);
                    self.push_msg("}");
                }
            }
        }

        self.push_msg("}");
    }
}
