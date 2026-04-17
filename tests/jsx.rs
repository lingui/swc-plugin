use lingui_macro_plugin::LinguiOptions;

#[macro_use]
mod common;

to!(
    jsx_simple_jsx,
    r#"
import { Trans } from "@lingui/react/macro";
const exp1 = <Custom>Refresh inbox</Custom>;
const exp2 = <Trans>Refresh inbox</Trans>;
const exp3 = <div><Trans>Refresh inbox</Trans></div>;
     "#
);

to!(
    jsx_should_suppor_legacy_import,
    r#"
import { Trans } from "@lingui/macro";
const exp2 = <Trans>Refresh inbox</Trans>;
     "#
);

to!(
    jsx_with_custom_id,
    r#"
       import { Trans } from "@lingui/react/macro";
       const exp2 = <Trans id="custom.id">Refresh inbox</Trans>;
     "#
);

to!(
    jsx_with_context,
    r#"
       import { Trans } from "@lingui/react/macro";
       const exp1 = <Trans>Refresh inbox</Trans>;
       const exp2 = <Trans context="My Context">Refresh inbox</Trans>;
     "#
);

to!(
    jsx_preserve_reserved_attrs,
    r#"
       import { Trans } from "@lingui/react/macro";
       const exp2 = <Trans comment="Translators Comment" context="Message Context" i18n="i18n" component={(p) => <div>{p.translation}</div>} render={(v) => v}>Refresh inbox</Trans>;
     "#
);

to!(
    jsx_expressions_are_converted_to_positional_arguments,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>
          Property {props.name},
          function {random()},
          array {array[index]},
          constant {42},
          object {new Date()},
          everything {props.messages[index].value()}
        </Trans>;
     "#
);

to!(
    jsx_comments_should_not_affect_expression_index,
    r#"
        import { Trans } from '@lingui/react/macro';
        // Without comment - expression gets index 0
        <Trans>
          Click here
          <Link>
            {getText()}
          </Link>
        </Trans>;
        // With comment before expression - expression should STILL get index 0
        <Trans>
          Click here
          <Link>
            {/* @ts-expect-error */}
            {getText()}
          </Link>
        </Trans>;
     "#
);

to!(
    jsx_components_interpolation,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>
          Hello <strong>World!</strong><br />
          <p>
            My name is <a href="/about">{" "}
            <em>{name}</em></a>
          </p>
        </Trans>
     "#
);

to!(
    jsx_values_dedup,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>
          Hello {foo} and {foo}{" "}
          {bar}
        </Trans>
     "#
);

to!(
    jsx_explicit_labels_with_as_statement,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>Refresh {{foo} as unknown as string} inbox</Trans>;
       "#
);

to!(
    jsx_explicit_labels,
    r#"
       import { Trans } from "@lingui/react/macro";

       <Trans>Refresh {{foo}} inbox</Trans>;
       <Trans>Refresh {{foo: foo.bar}} inbox</Trans>;
       <Trans>Refresh {{foo: expr()}} inbox</Trans>;
       <Trans>Refresh {{foo: bar, baz: qux}} inbox</Trans>;
       <Trans>Refresh {{}} inbox</Trans>;
       <Trans>Refresh {{...spread}} inbox</Trans>;
     "#
);

to!(
    jsx_ph_labels,
    r#"
       import { Trans, ph } from "@lingui/react/macro";

       <Trans>Refresh {ph({foo})} inbox</Trans>;
       <Trans>Refresh {ph({foo: foo.bar})} inbox</Trans>;
       <Trans>Refresh {ph({foo: expr()})} inbox</Trans>;
       <Trans>Refresh {ph({foo: bar, baz: qux})} inbox</Trans>;
       <Trans>Refresh {ph({})} inbox</Trans>;
       <Trans>Refresh {ph({...spread})} inbox</Trans>;
     "#
);

to!(
    jsx_nested_labels,
    r#"
       import { Trans, ph } from "@lingui/react/macro";

       <Trans>Refresh <span>{{foo}}</span> inbox</Trans>;
       <Trans>Refresh <span>{ph({foo})}</span> inbox</Trans>;
     "#
);

to!(
    jsx_template_literal_in_children,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>{`Hello ${foo} and ${bar}`}</Trans>
     "#
);

to!(
    quoted_jsx_attributes_are_handled,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>Speak "friend"!</Trans>;
       <Trans id="custom-id">Speak "friend"!</Trans>;
     "#
);

to!(
    html_attributes_are_handled,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>
          <Text>This should work &nbsp;</Text>
        </Trans>;
     "#
);

to!(
    use_decoded_html_entities,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>&amp;</Trans>
     "#
);

to!(
    elements_inside_expression_container,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>{<span>Component inside expression container</span>}</Trans>;
     "#
);

to!(
    elements_without_children,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>{<br />}</Trans>;
     "#
);

// it's better to throw panic here
// to!(
//     jsx_spread_child_is_noop,
//      r#"
//         import { Trans } from "@lingui/react/macro";
//         <Trans>{...spread}</Trans>
//      "#,
//     r#"
//         import { Trans as Trans_ } from "@lingui/react";
//         <Trans_>{...spread}</Trans>
//     "#
// );

to!(
    strip_whitespace_around_arguments,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>
          Strip whitespace around arguments: '
          {name}
          '
        </Trans>
     "#
);

to!(
    strip_whitespace_around_tags_but_keep_forced_spaces,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>
          Strip whitespace around tags, but keep{" "}
          <strong>forced spaces</strong>
          !
        </Trans>
     "#
);

to!(
    keep_multiple_forced_newlines,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>
          Keep multiple{"\n"}
          forced{"\n"}
          newlines!
        </Trans>
     "#
);

to!(
    use_js_macro_in_jsx_attrs,
    r#"
        import { t } from '@lingui/core/macro';
        import { Trans } from '@lingui/react/macro';
        <Trans>Read <a href="/more" title={t`Full content of ${articleName}`}>more</a></Trans>
     "#
);

to!(
    use_js_plural_in_jsx_attrs,
    r#"
        import { plural } from '@lingui/core/macro';
        <a href="/about" title={plural(count, {
          one: "\# book",
          other: "\# books"
        })}>About</a>
     "#
);

to!(
    ignore_jsx_empty_expression,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>Hello {/* and I cannot stress this enough */} World</Trans>;
     "#
);

to!(
    production_only_essential_props_are_kept,
    LinguiOptions {
        strip_non_essential_fields: true,
        ..Default::default()
    },
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans
        id="msg.hello"
        render="render"
        i18n="i18n"
        context="My Context"
        comment="Hello World">Hello <strong>{name}</strong></Trans>
     "#
);

to!(
    strip_whitespaces_in_jsxtext_but_keep_in_jsx_expression_containers,
    r#"
      import { Trans } from "@lingui/react/macro";
        <Trans>
        {"Wonderful framework "}
        <a href="https://nextjs.org">Next.js</a>
        {" say hi. And "}
        <a href="https://nextjs.org">Next.js</a>
        {" say hi."}
      </Trans>
     "#
);
to!(
    non_breaking_whitespace_handling_2226,
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  <span>hello</span>
  &nbsp;
  <span>world</span>
</Trans>;

     "#
);

to!(
    normalize_crlf_lf_cr,
    concat!(
        "import { Trans } from \"@lingui/react/macro\";\n",
        "<Trans>\nhello\n</Trans>;\n",
        "<Trans>\r\nhello\r\n</Trans>;\n",
        "<Trans>\rhello\r</Trans>;\n"
    )
);
//   {
//     name: "production - import_type_doesn't_interference_on_normal_import",
//     production: true,
//     useTypescriptPreset: true,
//     input: `
//         import { withI18nProps } from '@lingui/react'
//         import { Trans } from "@lingui/react/macro";
//         <Trans id="msg.hello" comment="Hello World">Hello World</Trans>
//       `,
//     expected: `
//         import { withI18nProps, Trans } from "@lingui/react";
//         <Trans id="msg.hello" />;
//       `,
//   },

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
