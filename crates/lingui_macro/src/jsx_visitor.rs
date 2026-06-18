use crate::ast_utils::{get_jsx_attr, get_jsx_attr_value_as_string};
use crate::macro_utils::MacroCtx;
use crate::tokens::{CaseOrOffset, ChoiceCase, IcuChoice, MsgToken, TagOpening};
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::visit::{Visit, VisitWith};
use swc_core::plugin::errors::HANDLER;

pub struct TransJSXVisitor<'a> {
    pub tokens: Vec<MsgToken>,
    ctx: &'a MacroCtx,
}

impl<'a> TransJSXVisitor<'a> {
    pub fn new(ctx: &'a MacroCtx) -> TransJSXVisitor<'a> {
        TransJSXVisitor {
            tokens: Vec::new(),
            ctx,
        }
    }
}

// taken from babel repo -> packages/babel-types/src/utils/react/cleanJSXElementLiteralChild.ts
fn split_lines(value: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = value.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\r' {
            lines.push(&value[start..i]);
            if i + 1 < len && bytes[i + 1] == b'\n' {
                i += 1;
            }
            start = i + 1;
        } else if bytes[i] == b'\n' {
            lines.push(&value[start..i]);
            start = i + 1;
        }
        i += 1;
    }

    lines.push(&value[start..]);
    lines
}

fn clean_jsx_element_literal_child(value: &str) -> String {
    let mut last_non_empty_line = 0;

    let lines = split_lines(value);

    for (i, line) in lines.iter().enumerate() {
        if line.bytes().any(|b| b != b'\t' && b != b' ') {
            last_non_empty_line = i;
        }
    }

    let mut result = String::new();

    for (i, line) in lines.iter().enumerate() {
        let is_first_line = i == 0;
        let is_last_line = i == lines.len() - 1;
        let is_last_non_empty_line = i == last_non_empty_line;

        // replace rendered whitespace tabs with spaces
        let mut trimmed_line = line.replace('\t', " ");

        // trim whitespace touching a newline
        if !is_first_line {
            trimmed_line = trimmed_line.trim_start_matches(' ').to_string();
        }

        // trim whitespace touching an endline
        if !is_last_line {
            trimmed_line = trimmed_line.trim_end_matches(' ').to_string();
        }

        if !trimmed_line.is_empty() {
            if !is_last_non_empty_line {
                trimmed_line.push(' ');
            }

            result.push_str(&trimmed_line);
        }
    }

    result
}

fn is_allowed_plural_option(key: &str) -> Option<Atom> {
    match key {
        "zero" | "one" | "two" | "few" | "many" | "other" => Some(key.into()),
        _ if key.starts_with('_') && key.len() > 1 => {
            let suffix = &key[1..];
            if suffix.bytes().all(|b| b.is_ascii_digit()) {
                Some(format!("={suffix}").into())
            } else if suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                Some(suffix.into())
            } else {
                None
            }
        }
        _ => None,
    }
}

impl TransJSXVisitor<'_> {
    // <Plural /> <Select /> <SelectOrdinal />
    fn visit_icu_macro(
        &mut self,
        el: &JSXOpeningElement,
        icu_format: &str,
    ) -> (Vec<CaseOrOffset>, usize) {
        let mut choices: Vec<CaseOrOffset> = Vec::new();
        // Default to the front so a missing `value` attribute keeps the prior
        // behaviour of allocating the value placeholder before any cases.
        let mut value_pos = 0;

        for attr in &el.attrs {
            if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                if let Some(attr_value) = &attr.value {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        if &ident.sym == "value" {
                            // Remember where the value sits relative to the
                            // cases so its index is allocated in source order.
                            value_pos = choices.len();
                        } else if &ident.sym == "offset" && icu_format != "select" {
                            if let Some(value) = get_jsx_attr_value_as_string(attr_value) {
                                choices.push(CaseOrOffset::Offset(value.to_string()))
                            } else {
                                // todo: panic offset might be only a number, other forms are not supported
                            }
                        } else if let Some(key) = is_allowed_plural_option(&ident.sym) {
                            let mut tokens: Vec<MsgToken> = Vec::new();

                            match attr_value {
                                // some="# books"
                                JSXAttrValue::Str(str) => {
                                    let string: String = str.value.to_string_lossy().into_owned();
                                    tokens.push(MsgToken::String(string));
                                }

                                JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                    expr: JSXExpr::Expr(exp),
                                    ..
                                }) => {
                                    match exp.as_ref() {
                                        // some={"# books"}
                                        Expr::Lit(Lit::Str(str)) => tokens.push(MsgToken::String(
                                            str.value.to_string_lossy().into_owned(),
                                        )),
                                        // some={`# books ${name}`}
                                        Expr::Tpl(tpl) => {
                                            tokens.extend(self.ctx.tokenize_tpl(tpl));
                                        }
                                        // some={`<Books />`}
                                        Expr::JSXElement(exp) => {
                                            let mut visitor = TransJSXVisitor::new(self.ctx);
                                            exp.visit_children_with(&mut visitor);

                                            tokens.extend(visitor.tokens)
                                        }

                                        _ => tokens.push(MsgToken::Expression(exp.clone())),
                                    }
                                }

                                _ => {
                                    // todo unsupported syntax
                                }
                            }

                            choices.push(CaseOrOffset::Case(ChoiceCase { tokens, key }))
                        }
                    }
                }
            } else {
                HANDLER.with(|h| {
                    h.struct_span_warn(el.span, "Unsupported Syntax")
                        .note("The spread expression could not be analyzed at compile time. Consider to use static values.")
                        .emit()
                });
            }
        }

        (choices, value_pos)
    }
}

impl Visit for TransJSXVisitor<'_> {
    fn visit_jsx_opening_element(&mut self, el: &JSXOpeningElement) {
        if let JSXElementName::Ident(ident) = &el.name {
            if self.ctx.is_lingui_ident("Trans", ident) {
                el.visit_children_with(self);
                return;
            }

            if self.ctx.is_lingui_jsx_choice_cmp(ident) {
                let value = match get_jsx_attr(el, "value").and_then(|attr| attr.value.as_ref()) {
                    Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        expr: JSXExpr::Expr(exp),
                        ..
                    })) => exp.clone(),
                    _ => Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
                };

                let icu_method = self
                    .ctx
                    .get_ident_export_name(ident)
                    .unwrap()
                    .to_lowercase();
                let (choices, value_pos) = self.visit_icu_macro(el, &icu_method);

                self.tokens.push(MsgToken::IcuChoice(IcuChoice {
                    cases: choices,
                    format: icu_method.into(),
                    value,
                    value_pos,
                }));

                return;
            }
        }

        self.tokens.push(MsgToken::TagOpening(TagOpening {
            self_closing: el.self_closing,
            el: JSXOpeningElement {
                self_closing: true,
                name: el.name.clone(),
                attrs: el.attrs.clone(),
                span: el.span,
                type_args: el.type_args.clone(),
            },
        }));
    }

    fn visit_jsx_closing_element(&mut self, _el: &JSXClosingElement) {
        self.tokens.push(MsgToken::TagClosing);
    }

    fn visit_jsx_text(&mut self, el: &JSXText) {
        self.tokens
            .push(MsgToken::String(clean_jsx_element_literal_child(&el.value)));
    }

    fn visit_jsx_expr_container(&mut self, cont: &JSXExprContainer) {
        if let JSXExpr::Expr(exp) = &cont.expr {
            match exp.as_ref() {
                Expr::Lit(Lit::Str(str)) => {
                    self.tokens
                        .push(MsgToken::String(str.value.to_string_lossy().into_owned()));
                }

                // todo write tests and validate
                // support calls to js macro inside JSX, but not to t``
                Expr::Call(call) => {
                    if let Some(tokens) = self.ctx.try_tokenize_call_expr_as_choice_cmp(call) {
                        self.tokens.extend(tokens);
                    } else if let Some(placeholder) =
                        self.ctx.try_tokenize_call_expr_as_placeholder_call(call)
                    {
                        self.tokens.push(placeholder);
                    } else {
                        self.tokens.push(MsgToken::Expression(exp.clone()));
                    }
                }

                Expr::JSXElement(jsx) => {
                    jsx.visit_children_with(self);
                }

                Expr::Tpl(tpl) => {
                    self.tokens.extend(self.ctx.tokenize_tpl(tpl));
                }
                _ => {
                    self.tokens.push(MsgToken::Expression(exp.clone()));
                }
            }
        }
    }
}
