use crate::ast_utils::{
    get_jsx_attr, get_jsx_attr_value_as_string, is_jsx_elements_equal, omit_jsx_attrs,
};
use crate::options::LinguiOptions;
use crate::tokens::{CaseOrOffset, MsgArg, MsgToken};
use std::collections::HashSet;
use swc_core::{common::DUMMY_SP, ecma::ast::*};

fn is_numeric(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn is_valid_placeholder_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let bytes = s.as_bytes();
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }

    if bytes.len() == 1 {
        return true;
    }

    let last = bytes[bytes.len() - 1];
    if !(last.is_ascii_alphanumeric() || last == b'_') {
        return false;
    }

    bytes[1..bytes.len() - 1]
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'-')
}

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
            options,
            elements_tracking: Vec::new(),
            element_index: 0,
        };

        builder.process_tokens(tokens);
        builder.into_args()
    }

    pub fn into_args(self) -> MessageBuilderResult {
        let message_str = self.message;

        let message = Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: message_str.clone().into(),
            raw: None,
        })));

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

                MsgToken::Arg(arg) => {
                    self.push_arg(arg);
                }

                MsgToken::TagOpening(val) => {
                    self.push_tag_opening(val.el, val.self_closing);
                }
                MsgToken::TagClosing => {
                    self.push_tag_closing();
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
            let attr = get_jsx_attr(&el, attr_name);

            let attr_value =
                attr.and_then(|attr| get_jsx_attr_value_as_string(attr.value.as_ref()?));

            if attr.is_some() && attr_value.is_none() {
                swc_core::plugin::errors::HANDLER.with(|h| {
                    h.struct_span_err(
                        el.span,
                        &format!("The `{attr_name}` attribute must be a non-empty string literal."),
                    )
                    .emit();
                });
            }

            base_name = attr_value;

            el.attrs = omit_jsx_attrs(el.attrs, HashSet::from([attr_name.as_str()]));
        }

        if base_name.is_none() {
            if let Some(defaults) = &self.options.jsx_placeholder_defaults {
                if let JSXElementName::Ident(ident) = &el.name {
                    if let Some(def) = defaults.get(&ident.sym.to_string()) {
                        base_name = Some(def.into());
                    }
                }
            }
        }

        let name = if let Some(n) = base_name {
            if is_numeric(&n) {
                swc_core::plugin::errors::HANDLER.with(|h| {
                    h.struct_span_err(
                        el.span,
                        &format!("Placeholder name `{n}` is not allowed because it conflicts with auto-generated numeric placeholders. Use a non-numeric name instead."),
                    ).emit();
                });
            } else if !is_valid_placeholder_name(&n) {
                swc_core::plugin::errors::HANDLER.with(|h| {
                    h.struct_span_err(
                        el.span,
                        &format!("Placeholder name `{n}` is not valid. Names must start and end with a letter/digit/underscore, but may contain `.-` in between."),
                    ).emit();
                });
            }

            if let Some((_, orig_el)) = self.elements_tracking.iter().find(|(k, _)| k == &n) {
                if !is_jsx_elements_equal(&el, orig_el) {
                    swc_core::plugin::errors::HANDLER.with(|h| {
                        let attr_name = self.options.jsx_placeholder_attribute.as_deref().unwrap_or("_t");
                        let eg = format!("(e.g. `<element {attr_name}=\"newName\" />`)");
                        let msg = format!(
                            "Multiple distinct JSX elements with the same placeholder name (`{n}`). Differentiate them by {} {eg}.",
                            if self.options.jsx_placeholder_attribute.is_some() {
                                format!("adding/modifying the `{attr_name}` attribute")
                            } else {
                                "setting `macro.jsxPlaceholderAttribute` in the lingui config and then adding the attribute to your JSX elements".to_string()
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

    fn push_arg(&mut self, arg: MsgArg) {
        let placeholder = arg.name.clone();

        self.values.push(ValueWithPlaceholder {
            placeholder: placeholder.clone(),
            value: arg.value,
        });

        if let Some(format) = arg.format {
            self.push_msg(&format!("{{{placeholder}, {format},"));

            if let Some(cases) = arg.cases {
                for choice in cases {
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
            }

            self.push_msg("}");
        } else {
            self.push_msg(&format!("{{{placeholder}}}"));
        }
    }
}
