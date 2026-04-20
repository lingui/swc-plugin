use lingui_macro_plugin::LinguiOptions;

#[macro_use]
mod common;

to!(
    jsx_named_placeholders_basic,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  Hello <strong _t="em">world</strong>!
</Trans>;
     "#
);

to!(
    jsx_named_placeholders_stripped_ast,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  <a _t="link" href="/about">About</a>
</Trans>;
     "#
);

to!(
    jsx_named_placeholders_defaults,
    LinguiOptions {
        jsx_placeholder_defaults: Some(std::collections::HashMap::from([
            ("a".into(), "link".into()),
            ("em".into(), "em".into()),
        ])),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  Here's a <a>link</a> and <em>emphasis</em>.
</Trans>;
     "#
);

to!(
    jsx_named_placeholders_mixed_explicit_and_defaults,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        jsx_placeholder_defaults: Some(std::collections::HashMap::from([(
            "a".into(),
            "link".into()
        ),])),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello <a href="/a">link 1</a>, normal, <a _t="link2" href="/b">link 2</a>.</Trans>;
     "#
);

to_panic!(
    jsx_named_placeholders_deduplication_different_props,
    LinguiOptions {
        jsx_placeholder_defaults: Some(std::collections::HashMap::from(
            [("a".into(), "a".into()),]
        )),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello <a href="/a">link 1</a>, normal, <a href="/b">link 2</a>.</Trans>;
     "#
);

to!(
    jsx_named_placeholders_deduplication_identical,
    LinguiOptions {
        jsx_placeholder_defaults: Some(std::collections::HashMap::from([(
            "em".into(),
            "em".into()
        ),])),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello <em>emphasis</em>, normal, <em>more emphasis</em>.</Trans>;
     "#
);

to_panic!(
    jsx_named_placeholders_deduplication_with_stripped_props,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello <a _t="link" href="/a">link 1</a>, normal, <a _t="link" href="/a">link 1 copy</a> and <a _t="link" href="/b">link 2</a>.</Trans>;
     "#
);

to!(
    jsx_named_placeholders_attribute_ignored_when_not_configured,
    LinguiOptions {
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  Hello <strong _t="em">world</strong>!
</Trans>;
     "#
);

to!(
    jsx_named_placeholders_prop_order,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello <a _t="link" href="/a" class="foo">link 1</a>, normal, <a _t="link" class="foo" href="/a">link 1 copy</a>.</Trans>;
     "#
);

to_panic!(
    jsx_named_placeholders_prop_order2,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello <a _t="link" href="/a" class="foo">link 1</a>, normal, <a _t="link" href="/b" class="foo">link 1 copy</a>.</Trans>;
     "#
);

to_panic!(
    jsx_named_placeholders_throws_on_non_string_attribute_value,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
const name = "link";
<Trans><a _t={name} href="/">click</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_throws_on_empty_attribute_value,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="" href="/">click</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_throws_on_numeric_name,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="0" href="/">click</a></Trans>
     "#
);

to!(
    jsx_named_placeholders_allows_hyphenated,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="foo-bar" href="/">click</a></Trans>
     "#
);

to!(
    jsx_named_placeholders_allows_dotted,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="ns.link" href="/">click</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_throws_starting_with_hyphen,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="-foo" href="/">click</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_throws_ending_with_dot,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="foo." href="/">click</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_same_name_different_element_throws,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><em _t="same">A</em> and <strong _t="same">B</strong></Trans>
     "#
);

to!(
    jsx_named_placeholders_identical_spreads_reused,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="same" {...spread}>A</a> <a _t="same" {...spread}>B</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_different_spreads_throw,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="same" {...spread1}>A</a> <a _t="same" {...spread2}>B</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_same_spread_different_order_throws,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="same" href="/" {...spread}>A</a> <a _t="same" {...spread} href="/">B</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_throws_on_empty_string,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t="" href="/">click</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_throws_on_jsx_expr,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t={"foo"} href="/">click</a></Trans>
     "#
);

to_panic!(
    jsx_named_placeholders_throws_on_boolean_expr,
    LinguiOptions {
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from '@lingui/react/macro';
<Trans><a _t href="/">click</a></Trans>
     "#
);
