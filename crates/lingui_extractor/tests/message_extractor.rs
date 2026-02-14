use lingui_extractor::extract_messages;
use lingui_extractor::ExtractedMessage;

fn extract_and_sort(source_code: &str, filename: &str) -> (Vec<ExtractedMessage>, Vec<String>) {
    let result = extract_messages(source_code, filename).expect("Failed to extract messages");
    (result.messages, result.warnings)
}

fn assert_no_warnings(warnings: &[String]) {
    if !warnings.is_empty() {
        panic!("Expected no warnings but got: {warnings:?}");
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
    assert!(messages.is_empty());
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
    assert!(!warnings.is_empty());
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
    assert!(!warnings.is_empty());
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

// #[test]
// fn test_origin_information() {
//     let code = r#"
// const msg = i18n._("Message");
//     "#;
//
//     let (messages, _) = extract_and_sort(code, "test.js");
//     assert_eq!(messages.len(), 1);
//
//     let origin = messages[0].origin.as_ref().unwrap();
//     assert_eq!(origin.0, "test.js");
//     assert_eq!(origin.1, 2); // Line 2 because of the blank line at the start
//     assert_eq!(origin.2, Some(13)); // Column where i18n._ starts
// }

// ============================================================================
// Snapshot Testing Framework
// ============================================================================

use std::fs;
use std::path::{Path, PathBuf};

/// Load fixture file from tests/fixtures/
fn load_fixture(filename: &str) -> String {
    let fixture_path = PathBuf::from("tests/fixtures").join(filename);
    fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path.display(), e))
}

/// Get snapshot path for a fixture
fn get_snapshot_path(fixture_name: &str) -> PathBuf {
    let snapshot_name = format!("{}.json", fixture_name);
    PathBuf::from("tests/__snapshots__").join(snapshot_name)
}

/// Load snapshot from disk
fn load_snapshot(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// Save snapshot to disk
fn save_snapshot(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap())
        .unwrap_or_else(|e| panic!("Failed to create snapshots directory: {}", e));
    fs::write(path, content)
        .unwrap_or_else(|e| panic!("Failed to write snapshot {}: {}", path.display(), e));
}

/// Check if UPDATE=1 environment variable is set
fn should_update_snapshots() -> bool {
    std::env::var("UPDATE").unwrap_or_default() == "1"
}

/// Serialize messages to JSON
fn serialize_to_json(messages: &[ExtractedMessage]) -> String {
    serde_json::to_string_pretty(messages).expect("Failed to serialize messages to JSON")
}

/// Perform snapshot test
fn snapshot_test(fixture_name: &str) {
    // Load fixture
    let source_code = load_fixture(fixture_name);

    // Extract messages
    let (messages, warnings) = extract_and_sort(&source_code, fixture_name);

    // Fail if there are warnings (optional - you can remove this if warnings are expected)
    if !warnings.is_empty() {
        eprintln!("Warnings during extraction from {}:", fixture_name);
        for warning in &warnings {
            eprintln!("  - {}", warning);
        }
    }

    // Serialize to JSON
    let actual_json = serialize_to_json(&messages);

    // Get snapshot path
    let snapshot_path = get_snapshot_path(fixture_name);

    if should_update_snapshots() {
        // Update mode: save the snapshot
        save_snapshot(&snapshot_path, &actual_json);
        println!("Updated snapshot: {}", snapshot_path.display());
    } else {
        // Compare mode: check against existing snapshot
        let expected_json = load_snapshot(&snapshot_path).unwrap_or_else(|| {
            panic!(
                "Snapshot file not found: {}\n\nTo create snapshots, run:\n  UPDATE=1 cargo test",
                snapshot_path.display()
            )
        });

        // Compare JSON
        if actual_json != expected_json {
            // Pretty print the difference
            eprintln!("\n❌ Snapshot mismatch for {}\n", fixture_name);
            eprintln!("Expected ({}):", snapshot_path.display());
            eprintln!("{}\n", expected_json);
            eprintln!("Actual:");
            eprintln!("{}\n", actual_json);
            eprintln!(
                "To update the snapshot, run:\n  UPDATE=1 cargo test {}",
                fixture_name.replace(".js", "").replace("-", "_")
            );
            panic!("Snapshot mismatch");
        }
    }
}

// ============================================================================
// Snapshot Tests
// ============================================================================

#[test]
fn test_snapshot_js_call_expression() {
    snapshot_test("js-call-expression.js");
}

#[test]
fn test_snapshot_js_message_descriptor() {
    snapshot_test("js-message-descriptor.js");
}

#[test]
fn test_snapshot_js_with_macros() {
    snapshot_test("js-with-macros.js");
}

#[test]
fn test_snapshot_jsx_with_macros() {
    snapshot_test("jsx-with-macros.js");
}

#[test]
fn test_snapshot_jsx_without_macros() {
    snapshot_test("jsx-without-macros.js");
}

#[test]
fn test_snapshot_jsx_without_trans() {
    snapshot_test("jsx-without-trans.js");
}

#[test]
fn test_snapshot_without_lingui() {
    snapshot_test("without-lingui.js");
}

#[test]
fn test_snapshot_with_sourcemaps() {
    snapshot_test("with-sourcemaps.js");
}
