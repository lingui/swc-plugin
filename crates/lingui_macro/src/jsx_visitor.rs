use crate::ast_utils::get_jsx_attr_value_as_string;
use crate::macro_utils::{
    tokenize_expr_to_arg, tokenize_tpl, try_tokenize_call_expr_as_choice_cmp, MacroCtx,
};
use crate::tokens::{CaseOrOffset, ChoiceCase, MsgArg, MsgToken, TagOpening};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::Atom;
use swc_core::plugin::errors::HANDLER;

pub struct TransJSXVisitor<'a, 'ctx> {
    pub tokens: Vec<MsgToken>,
    ctx: &'a mut MacroCtx<'ctx>,
}

impl<'a, 'ctx> TransJSXVisitor<'a, 'ctx> {
    pub fn new(ctx: &'a mut MacroCtx<'ctx>) -> TransJSXVisitor<'a, 'ctx> {
        TransJSXVisitor {
            tokens: Vec::new(),
            ctx,
        }
    }

    pub fn visit_jsx_children(&mut self, children: &Vec<JSXElementChild>) {
        for child in children {
            match child {
                JSXElementChild::JSXText(el) => self.visit_jsx_text(el),
                JSXElementChild::JSXExprContainer(cont) => self.visit_jsx_expr_container(cont),
                JSXElementChild::JSXElement(el) => {
                    self.visit_jsx_element(el);
                }
                JSXElementChild::JSXFragment(frag) => {
                    self.visit_jsx_children(&frag.children);
                }
                _ => {}
            }
        }
    }

    pub fn visit_jsx_element(&mut self, el: &JSXElement) {
        self.visit_jsx_opening_element(&el.opening);
        if !el.opening.self_closing {
            self.visit_jsx_children(&el.children);
            self.visit_jsx_closing_element();
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

impl TransJSXVisitor<'_, '_> {
    // <Plural /> <Select /> <SelectOrdinal />
    fn visit_icu_macro(&mut self, el: &JSXOpeningElement, icu_format: &str) {
        let mut cases: Vec<CaseOrOffset> = Vec::new();
        let mut value_arg: Option<MsgArg> = None;

        for attr in &el.attrs {
            if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                if let Some(attr_value) = &attr.value {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        if &ident.sym == "value" {
                            if let JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                expr: JSXExpr::Expr(exp),
                                ..
                            }) = attr_value
                            {
                                let token_arg = tokenize_expr_to_arg(self.ctx, exp.clone());
                                value_arg = Some(token_arg)
                            }
                        } else if &ident.sym == "offset" && icu_format != "select" {
                            if let Some(value) = get_jsx_attr_value_as_string(attr_value) {
                                cases.push(CaseOrOffset::Offset(value.to_string()))
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
                                            tokens.extend(tokenize_tpl(self.ctx, tpl));
                                        }
                                        // some={<Books />}
                                        Expr::JSXElement(el) => {
                                            let mut visitor = TransJSXVisitor::new(self.ctx);
                                            visitor.visit_jsx_element(el);

                                            tokens.extend(visitor.tokens);
                                        }

                                        _ => {
                                            let token_arg =
                                                tokenize_expr_to_arg(self.ctx, exp.clone());
                                            tokens.push(MsgToken::Arg(token_arg));
                                        }
                                    }
                                }

                                _ => {
                                    // todo unsupported syntax
                                }
                            }

                            cases.push(CaseOrOffset::Case(ChoiceCase { tokens, key }));
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

        if let Some(arg) = value_arg {
            self.tokens.push(MsgToken::Arg(MsgArg {
                name: arg.name,
                value: arg.value,
                format: Some(icu_format.into()),
                cases: Some(cases),
            }));
        } else {
            HANDLER.with(|h| {
                h.struct_span_warn(el.span, "Incorrect Macro Usage")
                    .note("The macro element should have a `value` property")
                    .emit()
            });
        }
    }
}

impl TransJSXVisitor<'_, '_> {
    pub fn visit_jsx_opening_element(&mut self, el: &JSXOpeningElement) {
        if let JSXElementName::Ident(ident) = &el.name {
            if self.ctx.transform.is_lingui_ident("Trans", ident) {
                return;
            }

            if self.ctx.transform.is_lingui_jsx_choice_cmp(ident) {
                let icu_method = self
                    .ctx
                    .transform
                    .get_ident_export_name(ident)
                    .unwrap()
                    .to_lowercase();
                self.visit_icu_macro(el, &icu_method);
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

    fn visit_jsx_closing_element(&mut self) {
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

                // support calls to js macro inside JSX, but not to t``
                Expr::Call(call) => {
                    if let Some(tokens) = try_tokenize_call_expr_as_choice_cmp(self.ctx, call) {
                        self.tokens.extend(tokens);
                    } else {
                        let arg = tokenize_expr_to_arg(self.ctx, exp.clone());
                        self.tokens.push(MsgToken::Arg(arg));
                    }
                }

                Expr::JSXElement(jsx) => {
                    self.visit_jsx_element(jsx);
                }

                Expr::Tpl(tpl) => {
                    self.tokens.extend(tokenize_tpl(self.ctx, tpl));
                }
                _ => {
                    let arg = tokenize_expr_to_arg(self.ctx, exp.clone());
                    self.tokens.push(MsgToken::Arg(arg));
                }
            }
        }
    }
}
