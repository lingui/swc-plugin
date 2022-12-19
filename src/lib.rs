use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use swc_core::ecma::{
    transforms::testing::test,
    visit::{as_folder, VisitMut, Visit, VisitMutWith, VisitWith},
};
use std::collections::hash_map::DefaultHasher;

use swc_core::{
    common::{DUMMY_SP},
    ecma::{
        parser::{Syntax, TsConfig},
        ast::*,
        atoms::JsWord,
        utils::{ExprFactory},
        visit::{Fold, FoldWith},
    },
    plugin::{
        plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::parser::lexer::util::CharExt;
// use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

mod utils;

const LINGUI_T: &str = &"t";

fn is_lingui_fn(name: &str) -> bool {
    // todo: i didn't find a better way to create a constant hashmap
    match name {
        "plural" | "select" | "selectOrdinal" => true,
        _ => false,
    }
}

fn match_callee_name(call: &CallExpr, fn_name: &str) -> bool {
    match &call.callee {
        Callee::Expr(expr) => {
            if let Expr::Ident(ident) = expr.as_ref() {
                return &ident.sym == fn_name;
            }
        }
        _ => {}
    }

    false
}

struct ValueWithPlaceholder {
    placeholder: String,
    value: Box<Expr>,
}

impl ValueWithPlaceholder {
    fn to_prop(self) -> PropOrSpread {
        let ident = Ident::new(self.placeholder.into(), DUMMY_SP);

        PropOrSpread::Prop(Box::new(
            Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(ident),
                value: self.value,
            })
        ))
    }
}

pub struct TransformVisitor;

impl TransformVisitor {
    // Receive an expression which expected to be either simple variable (ident) or expression
    // If simple variable is detected os literal used as placeholder
    // If expression detected we use index as placeholder.
    fn get_value_with_placeholder(&self, expr: Box<Expr>, i: &usize) -> ValueWithPlaceholder {
        match expr.as_ref() {
            // `text {foo} bar`
            Expr::Ident(ident) => {
                ValueWithPlaceholder {
                    placeholder: ident.sym.to_string(),
                    value: expr,
                }
            }
            // everything else, e.q.
            // `text {executeFn()} bar`
            // `text {bar.baz} bar`
            _ => {
                // would be a positional argument
                let index_str = &i.to_string();
                ValueWithPlaceholder {
                    placeholder: index_str.into(),
                    value: expr,
                }
            }
        }
    }

    // Receive TemplateLiteral with variables and return plane string where
    // substitutions replaced to placeholders and variables extracted as separate Vec
    // `Hello ${username}!` ->  (msg: `Hello {username}!`, variables: {username})
    fn transform_tpl_to_msg_and_values(&self, tpl: &Tpl) -> (String, Vec<PropOrSpread>) {
        let mut message = String::new();
        let values: Vec<&ValueWithPlaceholder> = Vec::with_capacity(tpl.exprs.len());
        let mut props = Vec::with_capacity(values.len());

        for (i, tpl_element) in tpl.quasis.iter().enumerate() {
            message.push_str(&tpl_element.raw);

            if let Some(exp) = tpl.exprs.get(i) {
                let val = self.get_value_with_placeholder(exp.clone(), &i);
                message.push_str(&format!("{{{}}}", &val.placeholder));
                props.push(val.to_prop());
            }
        }

        (message, props)
    }

    fn create_i18n_fn_call(&self, callee_obj: Box<Expr>, message: &str, values: Vec<PropOrSpread>) -> CallExpr {
        return CallExpr {
            span: DUMMY_SP,
            callee: Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: callee_obj,
                prop: MemberProp::Ident(Ident::new("_".into(), DUMMY_SP)),
            }).as_callee(),
            args: vec![
                message.as_arg(),
                Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: values,
                }).as_arg(),
            ],
            type_args: None,
        };
    }

    // receive ObjectLiteral {few: "..", many: "..", other: ".."} and create ICU string in form:
    // {count, plural, few {..} many {..} other {..}}
    // If messages passed as TemplateLiterals with variables, it extracts variables into Vec
    // (msg: {count, plural, one `{name} has # friend` other `{name} has # friends`}, variables: {name})
    fn get_icu_from_choices_obj(&self, props: &Vec<PropOrSpread>, icu_value_ident: &JsWord, icu_method: &JsWord) -> (String, Vec<PropOrSpread>) {
        let mut icu_parts: Vec<String> = Vec::with_capacity(props.len());
        let mut all_values: Vec<PropOrSpread> = Vec::new();

        for prop_or_spread in props {
            if let PropOrSpread::Prop(prop) = prop_or_spread {
                if let Prop::KeyValue(prop) = prop.as_ref() {
                    if let PropName::Ident(ident) = &prop.key {
                        let mut push_part = |msg: &str| {
                            icu_parts.push(format!("{} {{{}}}", &ident.sym, msg));
                        };

                        // String Literal: "has # friend"
                        if let Expr::Lit(lit) = prop.value.as_ref() {
                            if let Lit::Str(str) = lit {
                                // one {has # friend}
                                push_part(&str.value);
                            }
                        }

                        // Template Literal: `${name} has # friend`
                        if let Expr::Tpl(tpl) = prop.value.as_ref() {
                            let (msg, values) = self.transform_tpl_to_msg_and_values(tpl);
                            all_values.extend(values);
                            push_part(&msg);
                        }
                    } else {
                        // todo panic
                    }
                    // icuParts.push_str(prop.key)
                } else {
                    // todo: panic here we could not parse anything else then KeyValue pair
                }
            } else {
                // todo: panic here, we could not parse spread
            }
        }

        let msg = format!("{{{}, {}, {}}}", icu_value_ident, icu_method, icu_parts.join(" "));

        (msg, all_values)
    }
}

// fn get_jsx_element_id(name: &JSXElementName) -> &str {
//     match name {
//         JSXElementName::Ident(ident) => {
//
//         }
//
//         JSXElementName::JSXMemberExpr(member) => {
//             member.obj
//         }
//
//         JSXElementName::JSXNamespacedName(exp) => {
//             return &format!("{}:{}", exp.ns, exp.name);
//         }
// }

struct TransJSXVisitor /*<'a>*/ {
    message: String,
    components: Vec<ValueWithPlaceholder>,
    components_stack: Vec<usize>,
    values: Vec<ValueWithPlaceholder>,
    cmp_index: usize,
    value_index: usize,
}

impl TransJSXVisitor {
    fn new() -> TransJSXVisitor {
        TransJSXVisitor {
            message: String::new(),
            components: Vec::new(),
            components_stack: Vec::new(),
            values: Vec::new(),
            cmp_index: 0,
            value_index: 0,
        }
    }
}

impl Visit for TransJSXVisitor {
    // todo: how to handle fragments?
    fn visit_jsx_opening_element(&mut self, el: &JSXOpeningElement) {
        if el.self_closing {
            self.message.push_str(&format!("<{}/>", self.cmp_index));
        } else {
            self.components_stack.push(self.cmp_index);
            self.message.push_str(&format!("<{}>", self.cmp_index));
        }

        // todo: it looks very dirty and bad to cloning this jsx values
        self.components.push(ValueWithPlaceholder {
            placeholder: self.cmp_index.to_string(),
            value: Box::new(Expr::JSXElement(
                Box::new(
                    JSXElement {
                        opening: JSXOpeningElement {
                            self_closing: true,
                            name: el.name.clone(),
                            attrs: el.attrs.clone(),
                            span: el.span.clone(),
                            type_args: el.type_args.clone(),
                        },
                        closing: None,
                        children: vec![],
                        span: DUMMY_SP,
                    }
                )
            )),
        });
        self.cmp_index = self.cmp_index + 1;
    }

    fn visit_jsx_closing_element(&mut self, el: &JSXClosingElement) {
        if let Some(index) = self.components_stack.pop() {
            self.message.push_str(&format!("</{index}>"));
        } else {
            // todo JSX tags mismatch. write tests for tags mismatch, swc should not crash in that case
        }
    }

    fn visit_jsx_text(&mut self, el: &JSXText) {
        self.message.push_str(&el.value);
    }

    fn visit_jsx_expr_container(&mut self, cont: &JSXExprContainer) {
        if let JSXExpr::Expr(exp) = &cont.expr {
            match exp.as_ref() {
                Expr::Ident(ident) => {
                    self.message.push_str(&format!("{{{}}}", ident.sym));
                    self.values.push(ValueWithPlaceholder {
                        placeholder: ident.sym.to_string(),
                        value: exp.clone(),
                    });
                }
                Expr::Lit(Lit::Str(str)) => {
                    self.message.push_str(&str.value);
                }
                _ => {
                    self.message.push_str(&format!("{{{}}}", self.value_index));
                    self.values.push(ValueWithPlaceholder {
                        placeholder: (self.value_index.to_string()),
                        value: exp.clone(),
                    });

                    self.value_index = self.value_index + 1;
                }
            }
        }
    }
}

fn create_jsx_attribute(name: &str, exp: Expr) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: name.into(),
            optional: false,
        }),
        value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(Box::new(exp)),
        })),
    })
}

impl Fold for TransformVisitor {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        // if self.import_packages.is_empty() {
        //     return expr;
        // }
        if let Expr::TaggedTpl(tagged_tpl) = &expr {
            match tagged_tpl.tag.as_ref() {
                // t(i18n)``
                Expr::Call(call) if match_callee_name(call, LINGUI_T) => {
                    if let Some(v) = call.args.get(0) {
                        let (message, values)
                            = self.transform_tpl_to_msg_and_values(&tagged_tpl.tpl);
                        return Expr::Call(self.create_i18n_fn_call(
                            v.expr.clone(),
                            &message,
                            values,
                        ));
                    }
                }
                // t``
                Expr::Ident(ident) if &ident.sym == LINGUI_T => {
                    let (message, values)
                        = self.transform_tpl_to_msg_and_values(&tagged_tpl.tpl);

                    return Expr::Call(self.create_i18n_fn_call(
                        Box::new(Ident::new("i18n".into(), DUMMY_SP).into()),
                        &message,
                        values,
                    ));
                }
                _ => {}
            }
        }

        expr.fold_children_with(self)
    }

    fn fold_call_expr(&mut self, expr: CallExpr) -> CallExpr {
        // If no package that we care about is imported, skip the following
        // transformation logic.
        // if self.import_packages.is_empty() {
        //     return expr;
        // }

        if let Callee::Expr(e) = &expr.callee {
            match e.as_ref() {
                // (plural | select | selectOrdinal)()
                Expr::Ident(ident) => {
                    if !is_lingui_fn(&ident.sym) {
                        return expr;
                    }

                    if expr.args.len() != 2 {
                        // malformed plural call, exit
                        return expr;
                    }

                    // ICU value
                    let arg = expr.args.get(0).unwrap();
                    let icu_value
                        = self.get_value_with_placeholder(arg.expr.clone(), &0);

                    // ICU Choices
                    let arg = expr.args.get(1).unwrap();
                    if let Expr::Object(object) = &arg.expr.as_ref() {
                        let (message, values) = self.get_icu_from_choices_obj(
                            &object.props, &icu_value.placeholder.clone().into(), &ident.sym);

                        // todo need a function to remove duplicates from arguments
                        let mut all_values = vec![icu_value.to_prop()];
                        all_values.extend(values);

                        return self.create_i18n_fn_call(
                            Box::new(Ident::new("i18n".into(), DUMMY_SP).into()),
                            &message,
                            all_values,
                        );
                    } else {
                        // todo passed not an ObjectLiteral,
                        //      we should panic here or just skip this call
                    }
                }
                _ => {}
            }
        }

        expr
    }

    fn fold_jsx_element(&mut self, mut el: JSXElement) -> JSXElement {
        let mut msg: Option<&Atom> = None;

        if let JSXElementName::Ident(ident) = &el.opening.name {
            if &ident.sym != "Trans" {
                return el;
            }
        }

        let mut trans_visitor = TransJSXVisitor::new();

        el.children.visit_children_with(&mut trans_visitor);

        println!("{}", utils::normalize_whitespaces(&trans_visitor.message));

        let mut id: Option<&JsWord> = None;

        for el in &el.opening.attrs {
            if let JSXAttrOrSpread::JSXAttr(attr) = el {
                if let JSXAttrName::Ident(ident) = &attr.name {
                    if &ident.sym == "id" {
                        id = Some(&ident.sym)
                    }
                }
            } else {
                // todo panic unsupported syntax
            }
        }

        // for el in &el.children {
        //     if let JSXElementChild::JSXText(text) = el {
        //         msg = Some(&text.value);
        //     }
        // }

        // todo pass render prop to trans

        let mut attrs = vec![
            create_jsx_attribute(
                if let Some(_) = id { "message" } else { "id" }.into(),
                Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: utils::normalize_whitespaces(&trans_visitor.message).into(),
                    raw: None,
                })),
            ),
        ];

        if trans_visitor.values.len() > 0 {
            attrs.push(create_jsx_attribute(
                "values",
                Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: trans_visitor.values.into_iter().map(|item| item.to_prop()).collect(),
                }),
            ))
        }

        if trans_visitor.components.len() > 0 {
            attrs.push(create_jsx_attribute(
                "components",
                Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props: trans_visitor.components.into_iter().map(|item| item.to_prop()).collect(),
                }),
            ))
        }

        attrs.extend(el.opening.attrs);

        return JSXElement {
            span: el.span,
            children: vec![],
            closing: None,
            opening: JSXOpeningElement {
                self_closing: true,
                span: el.opening.span,
                name: el.opening.name,
                type_args: None,
                attrs,
            },
        };
    }
}


/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually
/// if plugin need to handle low-level ptr directly via
/// `__transform_plugin_process_impl(
///     ast_ptr: *const u8, ast_ptr_len: i32,
///     unresolved_mark: u32, should_enable_comments_proxy: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */`
///
/// This requires manual handling of serialization / deserialization from ptrs.
/// Refer swc_plugin_macro to see how does it work internally.
#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut TransformVisitor)
}

test!(
    Default::default(),
    |_| TransformVisitor,
    should_not_touch_not_related_tagget_tpls,
    // input
     r#"
     b`Refresh inbox`;
     b(i18n)`Refresh inbox`;
     "#,
    // output after transform
    r#"
    b`Refresh inbox`;
    b(i18n)`Refresh inbox`;
    "#
);

test!(
    Default::default(),
    |_| TransformVisitor,
    substitution_in_tpl_literal1,
    // input
     r#"
     t`Refresh inbox`
     t`Refresh ${foo} inbox ${bar}`
     t`Refresh ${foo.bar} inbox ${bar}`
     t`Refresh ${expr()}`
     "#,
    // output after transform
    r#"
    i18n._("Refresh inbox", {})
    i18n._("Refresh {foo} inbox {bar}", {foo: foo, bar: bar})
    i18n._("Refresh {0} inbox {bar}", {0: foo.bar, bar: bar})
    i18n._("Refresh {0}", {0: expr()})
    "#
);

test!(
    Default::default(),
    |_| TransformVisitor,
    custom_i18n_passed,
    // input
     r#"
     t(custom_i18n)`Refresh inbox`
     t(custom_i18n)`Refresh ${foo} inbox ${bar}`
     t(custom_i18n)`Refresh ${foo.bar} inbox ${bar}`
     t(custom_i18n)`Refresh ${expr()}`
     "#,
    // output after transform
    r#"
    custom_i18n._("Refresh inbox", {})
    custom_i18n._("Refresh {foo} inbox {bar}", {foo: foo, bar: bar})
    custom_i18n._("Refresh {0} inbox {bar}", {0: foo.bar, bar: bar})
    custom_i18n._("Refresh {0}", {0: expr()})
    "#
);

test!(
    Default::default(),
    |_| TransformVisitor,
    icu_functions,
     r#"
    const messagePlural = plural(count, {
       one: '# Book',
       other: '# Books'
    })
    const messageSelect = select(gender, {
       male: 'he',
       female: 'she',
       other: 'they'
    })
    const messageSelectOrdinal = selectOrdinal(count, {
       one: '#st',
       two: '#nd',
       few: '#rd',
       other: '#th',
    })
     "#,
    r#"
    const messagePlural = i18n._("{count, plural, one {# Book} other {# Books}}", {
      count: count
    });
    const messageSelect = i18n._("{gender, select, male {he} female {she} other {they}}", {
      gender: gender
    });
    const messageSelectOrdinal = i18n._("{count, selectOrdinal, one {#st} two {#nd} few {#rd} other {#th}}", {
      count: count
    });
    "#
);

test!(
    Default::default(),
    |_| TransformVisitor,
    should_not_touch_non_lungui_fns,
     r#"
    const messagePlural = customName(count, {
       one: '# Book',
       other: '# Books'
    })
     "#,
    r#"
   const messagePlural = customName(count, {
       one: '# Book',
       other: '# Books'
    })
    "#
);


test!(
    ignore, // todo need to implement dedupe of params
    Default::default(),
    |_| TransformVisitor,
    plural_with_placeholders,
     r#"
       const message = plural(count, {
           one: `${name} has # friend`,
           other: `${name} has # friends`
        })
     "#,
    r#"
    const message = i18n._("{count, plural, one {{name} has # friend} other {{name} has # friends}}", {
      count: count,
      name: name,
    })
    "#
);

// test!(
//     Default::default(),
//     |_| TransformVisitor,
//     plural_with_placeholders,
//      r#"
//       import { Trans } from "@lingui/macro"
//         <Trans>Refresh inbox</Trans>;
//      "#,
//     r#"
//    import { Trans } from "@lingui/react"
//     <Trans id="Refresh inbox" />
//     "#
// );

test!(
       Syntax::Typescript(TsConfig {
        tsx: true,
        ..Default::default()
    }),
    |_| TransformVisitor,
    simple_jsx,
     r#"
       const exp1 = <Custom>Refresh inbox</Custom>;
       const exp2 = <Trans>Refresh inbox</Trans>;
     "#,
    r#"
       const exp1 = <Custom>Refresh inbox</Custom>;
       const exp2 = <Trans id={"Refresh inbox"} />
    "#
);

test!(
       Syntax::Typescript(TsConfig {
        tsx: true,
        ..Default::default()
    }),
    |_| TransformVisitor,
    preserve_id_in_trans,
     r#"
       const exp2 = <Trans id="custom.id">Refresh inbox</Trans>;
     "#,
    r#"
       const exp2 = <Trans message={"Refresh inbox"} id="custom.id"/>
    "#
);

// todo whitespace management
// test!(
//        Syntax::Typescript(TsConfig {
//         tsx: true,
//         ..Default::default()
//     }),
//     |_| TransformVisitor,
//     jsx_interpolation,
//      r#"
//        <Trans>
//           Property {props.name},
//           function {random()},
//           array {array[index]},
//           constant {42},
//           object {new Date()},
//           everything {props.messages[index].value()}
//         </Trans>;
//      "#,
//     r#"
//        <Trans id={"Property {0}, function {1}, array {2}, constant {3}, object {4}, everything {5}"} values={{
//           0: props.name,
//           1: random(),
//           2: array[index],
//           3: 42,
//           4: new Date(),
//           5: props.messages[index].value()
//         }} />;
//     "#
// );

test!(
       Syntax::Typescript(TsConfig {
        tsx: true,
        ..Default::default()
    }),
    |_| TransformVisitor,
    jsx_components_interpolation,
     r#"
       <Trans>
          Hello <strong>World!</strong><br />
          <p>
            My name is <a href="/about">{" "}
            <em>{name} {expression()}</em></a>
          </p>
        </Trans>
     "#,
    r#"
       <Trans id={"Hello <0>World!</0><1/><2>My name is <3> <4>{name} {0}</4></3></2>"} values={{
          name: name,
          0: expression()
        }} components={{
          0: <strong />,
          1: <br />,
          2: <p />,
          3: <a href="/about" />,
          4: <em />
        }} />;
    "#
);
