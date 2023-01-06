use swc_core::ecma::{
    visit::{Visit, VisitWith},
};
use swc_core::ecma::ast::{*};
use swc_common::DUMMY_SP;
use crate::ast_utils::{get_jsx_attr, get_jsx_attr_value_as_string};
use crate::is_lingui_jsx_el;
use crate::tokens::{Icu, IcuChoice, IcuChoiceOrOffset, MsgToken, TagOpening};
use regex::{Regex};
use once_cell::sync::Lazy;
use swc_core::plugin::errors::HANDLER;
use crate::macro_utils::{tokenize_tpl, try_tokenize_call_expr_as_icu};

pub struct TransJSXVisitor {
    pub tokens: Vec<MsgToken>,
}

impl TransJSXVisitor {
    pub fn new() -> TransJSXVisitor {
        TransJSXVisitor {
            tokens: Vec::new(),
        }
    }
}

static PLURAL_OPTIONS_WHITELIST: Lazy<Regex> = Lazy::new(|| Regex::new(r"(_[\d\w]+|zero|one|two|few|many|other)").unwrap());
static NUM_OPTION: Lazy<Regex> = Lazy::new(|| Regex::new(r"_(\d+)").unwrap());
static WORD_OPTION: Lazy<Regex> = Lazy::new(|| Regex::new(r"_(\w+)").unwrap());

// const pluralRuleRe = /(_[\d\w]+|zero|one|two|few|many|other)/
// const jsx2icuExactChoice = (value: string) => value.replace(/_(\d+)/, "=$1").replace(/_(\w+)/, "$1")

fn is_allowed_plural_option(key: &str) -> Option<String> {
    if PLURAL_OPTIONS_WHITELIST.is_match(key) {
        let key = NUM_OPTION.replace(key, "=$1");
        let key = WORD_OPTION.replace(&key, "$1");

        return Some(key.to_string());
    }

    None
}

impl TransJSXVisitor {
    // <Plural /> <Select /> <SelectOrdinal />
    fn visit_icu_macro<'a>(&mut self, el: &JSXOpeningElement, icu_format: &str) -> Vec<IcuChoiceOrOffset> {
        let mut choices: Vec<IcuChoiceOrOffset> = Vec::new();

        for attr in &el.attrs {
            if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                if let Some(attr_value) = &attr.value {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        if &ident.sym == "offset" && icu_format != "select" {
                            if let Some(value) = get_jsx_attr_value_as_string(attr_value) {
                                choices.push(IcuChoiceOrOffset::Offset(value.to_string()))
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

                                JSXAttrValue::JSXExprContainer(JSXExprContainer { expr: JSXExpr::Expr(exp), .. }) => {
                                    match exp.as_ref() {
                                        // some={"# books"}
                                        Expr::Lit(Lit::Str(str)) => {
                                            tokens.push(MsgToken::String(str.value.clone().to_string()))
                                        }
                                        // some={`# books ${name}`}
                                        Expr::Tpl(tpl) => {
                                            tokens.extend(tokenize_tpl(tpl));
                                        }
                                        // some={`<Books />`}
                                        Expr::JSXElement(exp) => {
                                            let mut visitor = TransJSXVisitor::new();
                                            exp.visit_children_with(&mut visitor);

                                            tokens.extend(visitor.tokens)
                                        }

                                        _ => {
                                            // todo: unsupported syntax
                                        }
                                    }
                                }

                                _ => {
                                    // todo unsupported syntax
                                }
                            }

                            choices.push(IcuChoiceOrOffset::IcuChoice(
                                IcuChoice {
                                    tokens,
                                    key: key.to_string(),
                                }))
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

        return choices;
    }
}

impl Visit for TransJSXVisitor {
    fn visit_jsx_opening_element(&mut self, el: &JSXOpeningElement) {
        if let JSXElementName::Ident(ident) = &el.name {
            if &ident.sym == "Trans" {
                el.visit_children_with(self);
                return;
            }

            if is_lingui_jsx_el(&ident.sym) {
                let value = match get_jsx_attr(&el, "value").and_then(|attr| attr.value.as_ref()) {
                    Some(
                        JSXAttrValue::JSXExprContainer(
                            JSXExprContainer { expr: JSXExpr::Expr(exp), .. }
                        )
                    ) => {
                        exp.clone()
                    }
                    _ => {
                        Box::new(Expr::Lit(Lit::Null(Null {
                            span: DUMMY_SP
                        })))
                    }
                };

                let icu_method = ident.sym.to_lowercase();
                let choices = self.visit_icu_macro(el, &icu_method);

                self.tokens.push(MsgToken::Icu(Icu {
                    choices,
                    icu_method,
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
        self.tokens.push(
            MsgToken::TagClosing
        );
    }

    fn visit_jsx_text(&mut self, el: &JSXText) {
        self.tokens.push(
            MsgToken::String(el.value.to_string())
        );
    }

    fn visit_jsx_expr_container(&mut self, cont: &JSXExprContainer) {
        if let JSXExpr::Expr(exp) = &cont.expr {
            match exp.as_ref() {
                Expr::Lit(Lit::Str(str)) => {
                    self.tokens.push(
                        MsgToken::String(str.value.to_string())
                    );
                }

                // todo write tests and validate
                // support calls to js macro inside JSX, but not to t``
                Expr::Call(call) => {
                    if let Some(tokens) = try_tokenize_call_expr_as_icu(call) {
                        self.tokens.extend(tokens);
                    } else {
                        self.tokens.push(
                            MsgToken::Expression(exp.clone())
                        );
                    }
                }

                Expr::JSXElement(jsx) => {
                    jsx.visit_children_with(self);
                }

                Expr::Tpl(tpl) => {
                    self.tokens.extend(
                        tokenize_tpl(tpl)
                    );
                }
                _ => {
                    self.tokens.push(
                        MsgToken::Expression(exp.clone())
                    );
                }
            }
        }
    }
}


