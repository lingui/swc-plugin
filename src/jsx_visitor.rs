use swc_core::ecma::{
    visit::{Visit, VisitWith},
};
use swc_core::ecma::ast::{Expr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement, JSXElementName, JSXExpr, JSXExprContainer, JSXOpeningElement, JSXText, Lit, Null};
use swc_common::DUMMY_SP;
use crate::ecma_utils::{get_jsx_attr_value, match_jsx_name};
use crate::is_lingui_jsx_el;
use crate::tokens::{Icu, IcuChoice, MsgToken, TagOpening};

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

impl TransJSXVisitor {
    // <Plural /> <Select /> <SelectOrdinal />
    fn visit_icu_macro<'a>(&mut self, el: &JSXOpeningElement) -> Vec<IcuChoice> {
        let mut choices: Vec<IcuChoice> = Vec::new();

        for attr in &el.attrs {
            if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                if let Some(attr_value) = &attr.value {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        // todo: probably need blacklist more properties, or whitelist only selected
                        if (ident.sym.to_string() == "value") |
                            (ident.sym.to_string() == "id") {
                            continue;
                        }

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
                                    // Expr::Tpl(tpl) => {
                                    //     let (msg, values) = self.transform_tpl_to_msg_and_values(tpl);
                                    //     all_values.extend(values);
                                    //     push_part(&msg);
                                    // }
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

                        choices.push(IcuChoice {
                            tokens,
                            key: ident.sym.clone().to_string(),
                        })
                    }
                }
            } else {
                // todo here is spread which is not supported
            }
        }

        return choices;
    }
}

impl Visit for TransJSXVisitor {
    // todo: how to handle fragments?
    fn visit_jsx_opening_element(&mut self, el: &JSXOpeningElement) {
        if let JSXElementName::Ident(ident) = &el.name {
            if &ident.sym == "Trans" {
                println!("alive");
                el.visit_children_with(self);
                return;
            }

            if is_lingui_jsx_el(&ident.sym) {
                let value = match get_jsx_attr_value(&el, "value") {
                    Some(
                        JSXAttrValue::JSXExprContainer(
                            JSXExprContainer { expr: JSXExpr::Expr(exp), .. }
                        )
                    ) => {
                        exp.clone()
                    }
                    // todo: support here <Plural value=5 >
                    // JSXAttrValue::Lit(lit) => {
                    //     Box::new(Expr::Lit(*lit))
                    // }
                    _ => {
                        Box::new(Expr::Lit(Lit::Null(Null {
                            span: DUMMY_SP
                        })))
                    }
                };

                let choices = self.visit_icu_macro(el);

                self.tokens.push(MsgToken::Icu(Icu {
                    choices,
                    icu_method: ident.sym.to_lowercase(),
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
                _ => {
                    self.tokens.push(
                        MsgToken::Value(exp.clone())
                    );
                }
            }
        }
    }
}


