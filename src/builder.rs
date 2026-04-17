use crate::options::LinguiOptions;
use crate::ast_utils::expand_ts_as_expr;
use crate::tokens::{CaseOrOffset, IcuChoice, MsgToken};
use std::collections::HashSet;
use swc_core::{
    common::{SyntaxContext, EqIgnoreSpan, DUMMY_SP},
    ecma::ast::*,
};

static NUMERIC_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^\d+$").unwrap());
static VALID_NAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z_]([\w.-]*\w)?$").unwrap());

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
    pub fn into_prop(self) -> PropOrSpread {
        let key = if self.placeholder.contains('-') || self.placeholder.contains('.') {
            PropName::Computed(ComputedPropName {
                span: DUMMY_SP,
                expr: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: self.placeholder.clone().into(),
                    raw: None,
                }))),
            })
        } else {
            PropName::Ident(IdentName::new(self.placeholder.into(), DUMMY_SP))
        };

        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key,
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

pub struct MessageBuilder<'a> {
    message: String,

    components_stack: Vec<String>,
    components: Vec<ValueWithPlaceholder>,

    values: Vec<ValueWithPlaceholder>,
    values_indexed: Vec<ValueWithPlaceholder>,

    options: &'a LinguiOptions,
    elements_tracking: Vec<(String, JSXOpeningElement)>,
    element_index: usize,
}

impl<'a> MessageBuilder<'a> {
    pub fn parse(tokens: Vec<MsgToken>, options: &'a LinguiOptions) -> MessageBuilderResult {
        let mut builder = MessageBuilder {
            message: String::new(),
            components_stack: Vec::new(),
            components: Vec::new(),
            values: Vec::new(),
            values_indexed: Vec::new(),
            options,
            elements_tracking: Vec::new(),
            element_index: 0,
        };

        builder.process_tokens(tokens);
        builder.into_args()
    }

    pub fn into_args(mut self) -> MessageBuilderResult {
        let message_str = self.message;

        let message = Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: message_str.clone().into(),
            raw: None,
        })));

        self.values.append(&mut self.values_indexed);

        let values = if self.values.is_empty() {
            None
        } else {
            Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: dedup_values(self.values)
                    .into_iter()
                    .map(|item| item.into_prop())
                    .collect(),
            })))
        };

        let components = if self.components.is_empty() {
            None
        } else {
            Some(Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: self
                    .components
                    .into_iter()
                    .map(|item| item.into_prop())
                    .collect(),
            })))
        };

        MessageBuilderResult {
            message_str: message_str.to_string(),
            message,
            values,
            components,
        }
    }

    fn process_tokens(&mut self, tokens: Vec<MsgToken>) {
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

    fn push_tag_opening(&mut self, mut el: JSXOpeningElement, self_closing: bool) {
        let mut base_name: Option<String> = None;

        if let Some(attr_name) = &self.options.jsx_placeholder_attribute {
            if let Some(idx) = el.attrs.iter().position(|a| {
                if let JSXAttrOrSpread::JSXAttr(attr) = a {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        return &ident.sym == attr_name;
                    }
                }
                false
            }) {
                let attr = el.attrs.remove(idx);
                if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                    let mut is_valid = false;
                    if let Some(JSXAttrValue::Str(s)) = attr.value {
                        let val = s.value.to_string_lossy().into_owned();
                        if !val.is_empty() {
                            base_name = Some(val);
                            is_valid = true;
                        }
                    }
                    
                    if !is_valid {
                        swc_core::plugin::errors::HANDLER.with(|h| {
                            h.struct_span_err(
                                el.span,
                                &format!("The `{attr_name}` attribute must be a non-empty string literal."),
                            ).emit();
                        });
                    }
                }
            }
        }

        if base_name.is_none() {
            if let Some(defaults) = &self.options.jsx_placeholder_defaults {
                if let JSXElementName::Ident(ident) = &el.name {
                    if let Some(def) = defaults.get(&ident.sym.to_string()) {
                        base_name = Some(def.clone());
                    }
                }
            }
        }

        let name = if let Some(n) = base_name {
            if NUMERIC_REGEX.is_match(&n) {
                swc_core::plugin::errors::HANDLER.with(|h| {
                    h.struct_span_err(
                        el.span,
                        &format!("Placeholder name `{n}` is not allowed because it conflicts with auto-generated numeric placeholders. Use a non-numeric name instead."),
                    ).emit();
                });
            } else if !VALID_NAME_REGEX.is_match(&n) {
                swc_core::plugin::errors::HANDLER.with(|h| {
                    h.struct_span_err(
                        el.span,
                        &format!("Placeholder name `{n}` is not valid. Names must start and end with a letter/digit/underscore, but may contain `.-` in between."),
                    ).emit();
                });
            }

            if let Some((_, orig_el)) = self.elements_tracking.iter().find(|(k, _)| k == &n) {
                let has_spreads = orig_el.attrs.iter().any(|a| matches!(a, JSXAttrOrSpread::SpreadElement(_)));
                let attrs_equal = if orig_el.attrs.len() == el.attrs.len() {
                    if has_spreads {
                        orig_el.attrs.iter().zip(el.attrs.iter()).all(|(a, b)| a.eq_ignore_span(b))
                    } else {
                        orig_el.attrs.iter().all(|a| el.attrs.iter().any(|b| a.eq_ignore_span(b)))
                    }
                } else {
                    false
                };

                let tags_equal = el.name.eq_ignore_span(&orig_el.name);

                if !tags_equal || !attrs_equal {
                    swc_core::plugin::errors::HANDLER.with(|h| {
                        let attr_name = self.options.jsx_placeholder_attribute.as_deref().unwrap_or("_t");
                        let eg = format!("(e.g. `<element {attr_name}=\"newName\" />`)");
                        let msg = format!(
                            "Multiple distinct JSX elements with the same placeholder name (`{n}`). Differentiate them by {} {eg}.",
                            if self.options.jsx_placeholder_attribute.is_some() {
                                format!("adding/modifying the `{attr_name}` attribute")
                            } else {
                                format!("setting `macro.jsxPlaceholderAttribute` in the lingui config and then adding the attribute to your JSX elements")
                            }
                        );
                        h.struct_span_err(el.span, &msg).emit();
                    });
                }
            } else {
                self.elements_tracking.push((n.clone(), el.clone()));
            }

            n
        } else {
            let n = self.element_index.to_string();
            self.element_index += 1;
            self.elements_tracking.push((n.clone(), el.clone()));
            n
        };

        if self_closing {
            self.push_msg(&format!("<{name}/>"));
        } else {
            self.components_stack.push(name.clone());
            self.push_msg(&format!("<{name}>"));
        }

        if !self.components.iter().any(|c| c.placeholder == name) {
            // todo: it looks very dirty and bad to cloning this jsx values
            self.components.push(ValueWithPlaceholder {
                placeholder: name.clone(),
                value: Box::new(Expr::JSXElement(Box::new(JSXElement {
                    opening: el,
                    closing: None,
                    children: vec![],
                    span: DUMMY_SP,
                }))),
            });
        }
    }

    fn push_tag_closing(&mut self) {
        if let Some(name) = self.components_stack.pop() {
            self.push_msg(&format!("</{name}>"));
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
                    self.process_tokens(choice.tokens);
                    self.push_msg("}");
                }
            }
        }

        self.push_msg("}");
    }
}
