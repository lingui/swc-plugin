use lingui_macro_plugin::extract_messages;
use lingui_macro_plugin::message_extractor_visitor::ExtractedMessage;

fn extract_and_sort(source_code: &str, filename: &str) -> (Vec<ExtractedMessage>, Vec<String>) {
    let result = extract_messages(source_code, filename).expect("Failed to extract messages");
    (result.messages, result.warnings)
}

fn assert_no_warnings(warnings: &[String]) {
    if !warnings.is_empty() {
        panic!("Expected no warnings but got: {:?}", warnings);
    }
}

#[test]
fn test_ignore_files_without_lingui_import() {
    let code = r#"
const message = "Hello World";
console.log(message);
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_extract_from_jsx_trans_component() {
    let code = r#"
import { Trans } from "@lingui/react";

<Trans id="msg.hello" comment="Description" />;
<Trans id="msg.context" context="Context1" />;
<Trans id="msg.default" message="Hello World" />;
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 3);

    assert_eq!(messages[0].id, "msg.hello");
    assert_eq!(messages[0].comment, Some("Description".to_string()));

    assert_eq!(messages[1].id, "msg.context");
    assert_eq!(messages[1].context, Some("Context1".to_string()));

    assert_eq!(messages[2].id, "msg.default");
    assert_eq!(messages[2].message, Some("Hello World".to_string()));
}

#[test]
fn test_jsx_trans_no_warning_when_id_from_variable() {
    let code = r#"
import { Trans } from "@lingui/react";

<Trans id={message} />;
<Trans id={message.field} />;
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_jsx_trans_warning_when_missing_id() {
    let code = r#"
import { Trans } from "@lingui/react";

<Trans />;
<Trans message="Missing ID" />;
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_eq!(messages.len(), 0);
    assert!(!warnings.is_empty());
    assert!(warnings.iter().any(|w| w.contains("Missing message ID")));
}

#[test]
fn test_call_expression_i18n_underscore() {
    let code = r#"
const msg = i18n._("Message");
const withDescription = i18n._("Description", {}, { comment: "description" });
const withId = i18n._("ID", {}, { message: "Message with id" });
const withValues = i18n._("Values {param}", { param: param });
const withContext = i18n._("Some id", {}, { context: "Context1" });
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 5);

    assert_eq!(messages[0].id, "Message");

    assert_eq!(messages[1].id, "Description");
    assert_eq!(messages[1].comment, Some("description".to_string()));

    assert_eq!(messages[2].id, "ID");
    assert_eq!(messages[2].message, Some("Message with id".to_string()));

    assert_eq!(messages[3].id, "Values {param}");
    assert_eq!(
        messages[3].placeholders.get("param"),
        Some(&"param".to_string())
    );

    assert_eq!(messages[4].id, "Some id");
    assert_eq!(messages[4].context, Some("Context1".to_string()));
}

#[test]
fn test_call_expression_object_descriptor() {
    let code = r#"
i18n._({
  id: "my.id",
  message: "My Id Message",
  comment: "My comment",
});

// support alias
i18n.t("Aliased Message");

i18n.t({
  id: "my.id",
  message: "My Id Message",
  comment: "My comment",
});
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 3);

    assert_eq!(messages[0].id, "my.id");
    assert_eq!(messages[0].message, Some("My Id Message".to_string()));
    assert_eq!(messages[0].comment, Some("My comment".to_string()));

    assert_eq!(messages[1].id, "Aliased Message");

    assert_eq!(messages[2].id, "my.id");
}

#[test]
fn test_call_expression_member_access() {
    let code = r#"
// member access
ctx.i18n._("Message1");

// member access any depth
ctx.req.i18n._("Message2");
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, "Message1");
    assert_eq!(messages[1].id, "Message2");
}

#[test]
fn test_call_expression_ignore_non_i18n_members() {
    let code = r#"
i18n.load("Message");
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_call_expression_ignore_with_annotation() {
    let code = r#"
/* lingui-extract-ignore */
i18n._("Message1");

/* lingui-extract-ignore */
ctx.i18n._("Message2");

/* lingui-extract-ignore */
ctx.req.i18n._("Message3");
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_call_expression_no_warning_for_variables() {
    let code = r#"
i18n._(message);
// member expression
i18n._(foo.bar);
// function call
i18n._(getMessage());
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_call_expression_template_literal_and_concatenation() {
    let code = r#"
const msg = i18n._(`message.id`);
const msg2 = i18n._("second" + '.' + "id");
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, "message.id");
    assert_eq!(messages[1].id, "second.id");
}

#[test]
fn test_call_expression_string_concatenation_in_comment() {
    let code = r#"
const msg = i18n._('message.id', {}, {comment: "first " + "second " + "third"});
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].comment, Some("first second third".to_string()));
}

#[test]
fn test_string_literal_with_i18n_comment() {
    let code = r#"
const t = /*i18n*/'Message';
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, "Message");
}

#[test]
fn test_string_literal_empty_warning() {
    let code = r#"
const t = /*i18n*/'';
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_eq!(messages.len(), 0);
    assert!(warnings.len() > 0);
    assert!(warnings.iter().any(|w| w.contains("Empty StringLiteral")));
}

#[test]
fn test_message_descriptor_with_i18n_comment() {
    let code = r#"
const msg = /*i18n*/{id: "Message"};
const withDesc = /*i18n*/{id: "Description", comment: "description"};
const withId = /*i18n*/{id: "ID", message: "Message with id"};
const withContext = /*i18n*/{id: "Some id", context: "Context1"};
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 4);

    assert_eq!(messages[0].id, "Message");
    assert_eq!(messages[1].id, "Description");
    assert_eq!(messages[1].comment, Some("description".to_string()));
    assert_eq!(messages[2].id, "ID");
    assert_eq!(messages[2].message, Some("Message with id".to_string()));
    assert_eq!(messages[3].id, "Some id");
    assert_eq!(messages[3].context, Some("Context1".to_string()));
}

#[test]
fn test_message_descriptor_template_literal() {
    let code = r#"
const msg = /*i18n*/{id: `Message`};
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, "Message");
}

#[test]
fn test_message_descriptor_template_with_expressions_warning() {
    let code = r#"
const msg = /*i18n*/{id: `Hello ${name}`};
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert!(messages.is_empty());
    assert!(!warnings.is_empty());
    assert!(warnings
        .iter()
        .any(|w| w.contains("Could not extract from template literal with expressions")));
}

#[test]
fn test_message_descriptor_missing_id_warning() {
    let code = r#"
const msg = /*i18n*/ {message: `Hello ${name}`};
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_eq!(messages.len(), 0);
    assert!(warnings.len() > 0);
    assert!(warnings.iter().any(|w| w.contains("Missing message ID")));
}

#[test]
fn test_message_descriptor_string_concatenation() {
    let code = r#"
const msg =  /*i18n*/ {id: "msg.id", comment: "first " + "second " + "third"};
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].comment, Some("first second third".to_string()));
}

#[test]
fn test_placeholders_extraction() {
    let code = r#"
const msg = /*i18n*/{
  id: "Values {param} {name}",
  values: {
    param: param,
    name: "foo"
  }
};
    "#;

    let (messages, warnings) = extract_and_sort(code, "test.js");
    assert_no_warnings(&warnings);
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].placeholders.get("param"),
        Some(&"param".to_string())
    );
    assert_eq!(
        messages[0].placeholders.get("name"),
        Some(&"\"foo\"".to_string())
    );
}

#[test]
fn test_origin_information() {
    let code = r#"
const msg = i18n._("Message");
    "#;

    let (messages, _) = extract_and_sort(code, "test.js");
    assert_eq!(messages.len(), 1);

    let origin = messages[0].origin.as_ref().unwrap();
    assert_eq!(origin.0, "test.js");
    assert_eq!(origin.1, 2); // Line 2
}
