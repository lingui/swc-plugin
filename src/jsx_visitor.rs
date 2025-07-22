use crate::ast_utils::{get_jsx_attr, get_jsx_attr_value_as_string};
use crate::macro_utils::MacroCtx;
use crate::tokens::{Argument, CaseOrOffset, ChoiceCase, IcuChoice, MsgToken, TagOpening};
use once_cell::sync::Lazy;
use regex::Regex;
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

static PLURAL_OPTIONS_WHITELIST: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(_[\d\w]+|zero|one|two|few|many|other)").unwrap());
static NUM_OPTION: Lazy<Regex> = Lazy::new(|| Regex::new(r"_(\d+)").unwrap());
static WORD_OPTION: Lazy<Regex> = Lazy::new(|| Regex::new(r"_(\w+)").unwrap());

// const pluralRuleRe = /(_[\d\w]+|zero|one|two|few|many|other)/
// const jsx2icuExactChoice = (value: string) => value.replace(/_(\d+)/, "=$1").replace(/_(\w+)/, "$1")

static TRIM_START: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[ ]+").unwrap());
static TRIM_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ ]+$").unwrap());

// taken from babel repo -> packages/babel-types/src/utils/react/cleanJSXElementLiteralChild.ts
fn clean_jsx_element_literal_child(value: &str) -> String {
    let lines: Vec<&str> = value.split('\n').collect();
    let mut last_non_empty_line = 0;

    let re_non_space = Regex::new(r"[^\t ]").unwrap();

    for (i, line) in lines.iter().enumerate() {
        if re_non_space.is_match(line) {
            last_non_empty_line = i;
        }
    }

    let mut result = String::new();

    for (i, line) in lines.iter().enumerate() {
        let is_first_line = i == 0;
        let is_last_line = i == lines.len() - 1;
        let is_last_non_empty_line = i == last_non_empty_line;

        // replace rendered whitespace tabs with spaces
        let mut trimmed_line = line.replace("\t", " ");

        // trim whitespace touching a newline
        if !is_first_line {
            trimmed_line = TRIM_START.replace(&trimmed_line, "").to_string();
        }

        // trim whitespace touching an endline
        if !is_last_line {
            trimmed_line = TRIM_END.replace(&trimmed_line, "").to_string();
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
    if PLURAL_OPTIONS_WHITELIST.is_match(key) {
        let key = NUM_OPTION.replace(key, "=$1");
        let key = WORD_OPTION.replace(&key, "$1");

        return Some(key.to_string().into());
    }

    None
}

impl TransJSXVisitor<'_> {
    // <Plural /> <Select /> <SelectOrdinal />
    fn visit_icu_macro(&mut self, el: &JSXOpeningElement, icu_format: &str) -> Vec<CaseOrOffset> {
        let mut choices: Vec<CaseOrOffset> = Vec::new();

        for attr in &el.attrs {
            if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                if let Some(attr_value) = &attr.value {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        if &ident.sym == "offset" && icu_format != "select" {
                            if let Some(value) = get_jsx_attr_value_as_string(attr_value) {
                                choices.push(CaseOrOffset::Offset(value.to_string()))
                            } else {
                                // todo: panic offset might be only a number, other forms are not supported
                            }
                        } else if let Some(key) = is_allowed_plural_option(&ident.sym) {
                            let mut tokens: Vec<MsgToken> = Vec::new();

                            match attr_value {
                                // some="# books"
                                JSXAttrValue::Lit(Lit::Str(str)) => {
                                    let string: String = str.value.clone().to_string();
                                    tokens.push(MsgToken::String(string));
                                }

                                JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                    expr: JSXExpr::Expr(exp),
                                    ..
                                }) => {
                                    match exp.as_ref() {
                                        // some={"# books"}
                                        Expr::Lit(Lit::Str(str)) => tokens
                                            .push(MsgToken::String(str.value.clone().to_string())),
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

                                        _ => tokens.push(MsgToken::Argument(Argument {
                                            used_utility_name: None,
                                            value: exp.clone(),
                                        })),
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

        choices
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
                let choices = self.visit_icu_macro(el, &icu_method);

                self.tokens.push(MsgToken::IcuChoice(IcuChoice {
                    cases: choices,
                    format: icu_method.into(),
                    value,
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
                    self.tokens.push(MsgToken::String(str.value.to_string()));
                }

                // todo write tests and validate
                // support calls to js macro inside JSX, but not to t``
                Expr::Call(call) => {
                    if let Some(tokens) = self.ctx.try_tokenize_call_expr_as_choice_cmp(call) {
                        self.tokens.extend(tokens);
                    } else if let Some(arg_token) =
                        self.ctx.try_tokenize_call_expr_as_utility_macro_call(call)
                    {
                        self.tokens.push(arg_token);
                    } else {
                        self.tokens.push(MsgToken::Argument(Argument {
                            used_utility_name: None,
                            value: exp.clone(),
                        }));
                    }
                }

                Expr::JSXElement(jsx) => {
                    jsx.visit_children_with(self);
                }

                Expr::Tpl(tpl) => {
                    self.tokens.extend(self.ctx.tokenize_tpl(tpl));
                }
                _ => {
                    self.tokens.push(MsgToken::Argument(Argument {
                        used_utility_name: None,
                        value: exp.clone(),
                    }));
                }
            }
        }
    }
}
