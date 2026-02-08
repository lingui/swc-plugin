use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc as Lrc;
use swc_core::common::comments::Comments;
use swc_core::common::source_map::SmallPos;
use swc_core::common::{SourceMap, Span, Spanned};
use swc_core::ecma::ast::{
    BinaryOp, CallExpr, Callee, Expr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement,
    JSXElementName, JSXExpr, Lit, MemberProp, ObjectLit, Prop, PropName, PropOrSpread, Str,
};
use swc_core::ecma::visit::{Visit, VisitWith};

/// Represents the location where a message was found
pub type Origin = (String, usize, Option<usize>);

/// A message extracted from source code
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtractedMessage {
    pub id: String,
    pub message: Option<String>,
    pub context: Option<String>,
    pub comment: Option<String>,
    pub placeholders: BTreeMap<String, String>,
    pub origin: Option<Origin>,
}

/// Internal structure for building messages
#[derive(Debug, Default)]
struct RawMessage {
    id: Option<String>,
    message: Option<String>,
    comment: Option<String>,
    context: Option<String>,
    placeholders: Option<BTreeMap<String, String>>,
}

/// Result of message extraction containing messages and any warnings
#[derive(Debug)]
pub struct ExtractionResult {
    pub messages: Vec<ExtractedMessage>,
    pub warnings: Vec<String>,
}

const I18N_OBJECT: &str = "i18n";

/// Check if a node has a specific leading comment
fn has_comment(comments: &dyn Comments, span: Span, comment_text: &str) -> bool {
    if let Some(leading) = comments.get_leading(span.lo) {
        return leading.iter().any(|c| c.text.trim() == comment_text);
    }
    false
}

/// Check if a node has the lingui-extract-ignore comment
fn has_ignore_comment(comments: &dyn Comments, span: Span) -> bool {
    has_comment(comments, span, "lingui-extract-ignore")
}

/// Check if a node has the i18n marker comment
fn has_i18n_comment(comments: &dyn Comments, span: Span) -> bool {
    has_comment(comments, span, "i18n")
}

/// Extract text from an expression (handles strings, concatenation, template literals)
fn get_text_from_expression(
    expr: &Expr,
    emit_error_on_variable: bool,
    warnings: &mut Vec<String>,
) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(s)) => Some(s.value.to_string_lossy().into_owned()),

        Expr::Bin(bin_expr) if bin_expr.op == BinaryOp::Add => {
            let left = get_text_from_expression(&bin_expr.left, emit_error_on_variable, warnings);
            let right = get_text_from_expression(&bin_expr.right, emit_error_on_variable, warnings);

            left.zip(right)
                .map(|(left, right)| format!("{left}{right}"))
        }

        Expr::Tpl(tpl) => {
            if tpl.quasis.len() > 1 {
                warnings
                    .push("Could not extract from template literal with expressions.".to_string());
                return None;
            }

            tpl.quasis
                .first()
                .and_then(|q| q.cooked.as_ref().map(|s| s.to_string_lossy().into_owned()))
        }

        _ => {
            if emit_error_on_variable {
                warnings.push("Only strings or template literals could be extracted.".to_string());
            }
            None
        }
    }
}

/// Get source code text for a span
fn get_node_source(source_code: &str, span: Span) -> String {
    let start = span.lo.to_usize() - 1;
    let end = span.hi.to_usize() - 1;

    if start < source_code.len() && end <= source_code.len() && start < end {
        source_code[start..end].to_string()
    } else {
        String::new()
    }
}

/// Convert a values object expression to placeholders HashMap
fn values_object_to_placeholders(
    obj: &ObjectLit,
    source_code: &str,
    warnings: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let mut placeholders = BTreeMap::new();

    for (i, prop) in obj.props.iter().enumerate() {
        if let PropOrSpread::Prop(prop) = prop {
            if let Prop::KeyValue(kv) = &**prop {
                let name = match &kv.key {
                    PropName::Ident(id) => Some(id.sym.to_string()),
                    PropName::Str(s) => Some(s.value.to_string_lossy().into_owned()),
                    PropName::Num(n) => Some(n.value.to_string()),
                    _ => {
                        warnings.push(format!(
                            "Could not extract values to placeholders. The key #{i} has unsupported syntax",
                        ));
                        None
                    }
                };

                if let Some(name) = name {
                    // kv.value.
                    let value_source = get_node_source(source_code, kv.value.span());
                    placeholders.insert(name, value_source);
                }
            }
        }
    }

    placeholders
}

/// Extract message properties from an object expression
fn extract_from_object_expression(
    obj: &ObjectLit,
    source_code: &str,
    warnings: &mut Vec<String>,
) -> RawMessage {
    let mut raw_msg = RawMessage::default();

    let text_keys = ["id", "message", "comment", "context"];

    for prop in &obj.props {
        if let PropOrSpread::Prop(prop) = prop {
            if let Prop::KeyValue(kv) = &**prop {
                if let PropName::Ident(key_ident) = &kv.key {
                    let key_name = key_ident.sym.as_ref();

                    if key_name == "values" {
                        if let Expr::Object(obj_expr) = &*kv.value {
                            raw_msg.placeholders = Some(values_object_to_placeholders(
                                obj_expr,
                                source_code,
                                warnings,
                            ));
                        }
                    } else if text_keys.contains(&key_name) {
                        let text = get_text_from_expression(&kv.value, true, warnings);

                        match key_name {
                            "id" => raw_msg.id = text,
                            "message" => raw_msg.message = text,
                            "comment" => raw_msg.comment = text,
                            "context" => raw_msg.context = text,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    raw_msg
}

/// Visitor for extracting messages from AST
pub struct MessageExtractorVisitor<'a> {
    pub messages: Vec<ExtractedMessage>,
    pub warnings: Vec<String>,
    source_map: Lrc<SourceMap>,
    comments: &'a dyn Comments,
    source_code: String,
    filename: String,
}

impl<'a> MessageExtractorVisitor<'a> {
    pub fn new(
        source_map: Lrc<SourceMap>,
        comments: &'a dyn Comments,
        source_code: String,
        filename: String,
    ) -> Self {
        Self {
            messages: Vec::new(),
            warnings: Vec::new(),
            source_map,
            comments,
            source_code,
            filename,
        }
    }

    /// Collect a message and add it to the list
    fn collect_message(&mut self, raw: RawMessage, span: Span) {
        // Prevent from adding undefined msgid
        if raw.id.is_none() {
            return;
        }

        // Extract line and column from span using SourceMap
        let loc = self.source_map.lookup_char_pos(span.lo);

        // Check if column is valid (not a synthetic/dummy position from macros)
        // Synthetic spans often have very large column values that would overflow
        let col = if loc.col.0 < 1000000 {
            Some(loc.col.to_usize() + 1) // Convert to 1-based
        } else {
            None // Invalid/synthetic column, omit it
        };

        let origin = Some((
            self.filename.clone(),
            loc.line, // Accurate line number (1-based)
            col,      // Column number (1-based) or None if synthetic
        ));

        self.messages.push(ExtractedMessage {
            id: raw.id.unwrap(),
            message: raw.message,
            context: raw.context,
            comment: raw.comment,
            placeholders: raw.placeholders.unwrap_or_default(),
            origin,
        });
    }

    /// Check if a JSX element is a Trans component from @lingui/react
    fn is_trans_component(&self, el: &JSXElement) -> bool {
        // For now, we check if the name is "Trans"
        // In a full implementation, we would track imports
        if let JSXElementName::Ident(ident) = &el.opening.name {
            ident.sym.as_ref() == "Trans"
        } else {
            false
        }
    }

    /// Check if a callee is an i18n method (_ or t)
    fn is_i18n_method(&self, callee: &Callee) -> bool {
        if let Callee::Expr(expr) = callee {
            if let Expr::Member(member) = &**expr {
                // Check if property is _ or t
                let is_underscore_or_t = match &member.prop {
                    MemberProp::Ident(id) => id.sym.as_ref() == "_" || id.sym.as_ref() == "t",
                    _ => false,
                };

                if !is_underscore_or_t {
                    return false;
                }

                // Check if object is i18n or ends with .i18n
                match &*member.obj {
                    Expr::Ident(id) if id.sym.as_ref() == I18N_OBJECT => return true,
                    Expr::Member(inner_member) => {
                        if let MemberProp::Ident(id) = &inner_member.prop {
                            return id.sym.as_ref() == I18N_OBJECT;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Extract from a message descriptor (object expression with i18n comment)
    fn extract_from_message_descriptor(&mut self, obj: &ObjectLit, span: Span) {
        let raw = extract_from_object_expression(obj, &self.source_code, &mut self.warnings);

        if raw.id.is_none() {
            let loc = self.source_map.span_to_string(span);
            self.warnings
                .push(format!("{loc}: Missing message ID, skipping."));
            return;
        }

        self.collect_message(raw, span);
    }
}

impl<'a> Visit for MessageExtractorVisitor<'a> {
    fn visit_call_expr(&mut self, call: &CallExpr) {
        // Check for ignore comment
        if has_ignore_comment(self.comments, call.span) {
            call.visit_children_with(self);
            return;
        }

        // Check if this is i18n._ or i18n.t
        if !self.is_i18n_method(&call.callee) {
            call.visit_children_with(self);
            return;
        }

        let first_arg = call.args.first();
        if first_arg.is_none() {
            call.visit_children_with(self);
            return;
        }

        let first_arg = &first_arg.unwrap().expr;

        // Skip if first argument has i18n comment (will be processed by ObjectExpression visitor)
        if has_i18n_comment(self.comments, first_arg.span()) {
            call.visit_children_with(self);
            return;
        }

        // Check if first argument is an object expression
        if let Expr::Object(obj) = &**first_arg {
            self.extract_from_message_descriptor(obj, call.span);
            call.visit_children_with(self);
            return;
        }

        // Otherwise, extract id from first argument
        let id = get_text_from_expression(
            first_arg,
            false, // Don't emit error on variable
            &mut self.warnings,
        );

        if id.is_none() {
            call.visit_children_with(self);
            return;
        }

        let mut raw = RawMessage {
            id,
            ..Default::default()
        };

        // Extract placeholders from second argument if it's an object
        if let Some(second_arg) = call.args.get(1) {
            if let Expr::Object(obj) = &*second_arg.expr {
                raw.placeholders = Some(values_object_to_placeholders(
                    obj,
                    &self.source_code,
                    &mut self.warnings,
                ));
            }
        }

        // Merge with third argument if it's an object (message descriptor)
        if let Some(third_arg) = call.args.get(2) {
            if let Expr::Object(obj) = &*third_arg.expr {
                let descriptor =
                    extract_from_object_expression(obj, &self.source_code, &mut self.warnings);

                // Merge properties (keeping existing id)
                if descriptor.message.is_some() {
                    raw.message = descriptor.message;
                }
                if descriptor.comment.is_some() {
                    raw.comment = descriptor.comment;
                }
                if descriptor.context.is_some() {
                    raw.context = descriptor.context;
                }
            }
        }

        self.collect_message(raw, call.span);
        call.visit_children_with(self);
    }

    fn visit_jsx_element(&mut self, el: &JSXElement) {
        // Check if this is a Trans component
        if !self.is_trans_component(el) {
            el.visit_children_with(self);
            return;
        }

        // Check for spread attribute with i18n comment
        for attr in &el.opening.attrs {
            if let JSXAttrOrSpread::SpreadElement(spread) = attr {
                if has_i18n_comment(self.comments, spread.expr.span()) {
                    el.visit_children_with(self);
                    return;
                }
            }
        }

        // Extract props from attributes
        let mut raw = RawMessage::default();

        for attr in &el.opening.attrs {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(name_ident) = &jsx_attr.name {
                    let key = name_ident.sym.as_ref();

                    match key {
                        "id" | "message" | "comment" | "context" => {
                            let value = match &jsx_attr.value {
                                Some(JSXAttrValue::Str(s)) => {
                                    Some(s.value.to_string_lossy().into_owned())
                                }
                                Some(JSXAttrValue::JSXExprContainer(container)) => {
                                    if let JSXExpr::Expr(expr) = &container.expr {
                                        if let Expr::Lit(Lit::Str(s)) = &**expr {
                                            Some(s.value.to_string_lossy().into_owned())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            if let Some(v) = value {
                                match key {
                                    "id" => raw.id = Some(v),
                                    "message" => raw.message = Some(v),
                                    "comment" => raw.comment = Some(v),
                                    "context" => raw.context = Some(v),
                                    _ => {}
                                }
                            }
                        }
                        "values" => {
                            if let Some(JSXAttrValue::JSXExprContainer(container)) = &jsx_attr.value
                            {
                                if let JSXExpr::Expr(expr) = &container.expr {
                                    if let Expr::Object(obj) = &**expr {
                                        raw.placeholders = Some(values_object_to_placeholders(
                                            obj,
                                            &self.source_code,
                                            &mut self.warnings,
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check if id is missing
        if raw.id.is_none() {
            // Check if there's an id prop at all
            let has_id_prop = el.opening.attrs.iter().any(|attr| {
                if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                    if let JSXAttrName::Ident(name_ident) = &jsx_attr.name {
                        return name_ident.sym.as_ref() == "id";
                    }
                }
                false
            });

            // Only warn if there's no id prop or if the id value is a literal but empty
            if !has_id_prop {
                let loc = self.source_map.span_to_string(el.span);
                self.warnings
                    .push(format!("{loc}: Missing message ID, skipping."));
            }
            el.visit_children_with(self);
            return;
        }

        self.collect_message(raw, el.span);
        el.visit_children_with(self);
    }

    fn visit_object_lit(&mut self, obj: &ObjectLit) {
        // Check for i18n comment
        if has_i18n_comment(self.comments, obj.span) {
            self.extract_from_message_descriptor(obj, obj.span);
        }

        obj.visit_children_with(self);
    }

    fn visit_str(&mut self, s: &Str) {
        // Check for i18n comment
        if !has_i18n_comment(self.comments, s.span) {
            return;
        }

        let id = s.value.to_string_lossy().into_owned();

        if id.is_empty() {
            self.warnings
                .push("Empty StringLiteral, skipping.".to_string());
            return;
        }

        let raw = RawMessage {
            id: Some(id),
            ..Default::default()
        };

        self.collect_message(raw, s.span);
    }
}
