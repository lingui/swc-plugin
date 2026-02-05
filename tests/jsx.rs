#[macro_use]
mod common;

lingui_test!(
    jsx_simple_jsx,
    r#"
import { Trans } from "@lingui/react/macro";
const exp1 = <Custom>Refresh inbox</Custom>;
const exp2 = <Trans>Refresh inbox</Trans>;
const exp3 = <div><Trans>Refresh inbox</Trans></div>;
     "#
);

lingui_test!(
    jsx_should_suppor_legacy_import,
    r#"
import { Trans } from "@lingui/macro";
const exp2 = <Trans>Refresh inbox</Trans>;
     "#
);

lingui_test!(
    jsx_with_custom_id,
    r#"
       import { Trans } from "@lingui/react/macro";
       const exp2 = <Trans id="custom.id">Refresh inbox</Trans>;
     "#
);

lingui_test!(
    jsx_with_context,
    r#"
       import { Trans } from "@lingui/react/macro";
       const exp1 = <Trans>Refresh inbox</Trans>;
       const exp2 = <Trans context="My Context">Refresh inbox</Trans>;
     "#
);

lingui_test!(
    jsx_preserve_reserved_attrs,
    r#"
       import { Trans } from "@lingui/react/macro";
       const exp2 = <Trans comment="Translators Comment" context="Message Context" i18n="i18n" component={(p) => <div>{p.translation}</div>} render={(v) => v}>Refresh inbox</Trans>;
     "#
);

lingui_test!(
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

lingui_test!(
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

lingui_test!(
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

lingui_test!(
    jsx_values_dedup,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>
          Hello {foo} and {foo}{" "}
          {bar}
        </Trans>
     "#
);

lingui_test!(
    jsx_explicit_labels_with_as_statement,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>Refresh {{foo} as unknown as string} inbox</Trans>;
       "#
);

lingui_test!(
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

lingui_test!(
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

lingui_test!(
    jsx_nested_labels,
    r#"
       import { Trans, ph } from "@lingui/react/macro";

       <Trans>Refresh <span>{{foo}}</span> inbox</Trans>;
       <Trans>Refresh <span>{ph({foo})}</span> inbox</Trans>;
     "#
);

lingui_test!(
    jsx_template_literal_in_children,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>{`Hello ${foo} and ${bar}`}</Trans>
     "#
);

lingui_test!(
    quoted_jsx_attributes_are_handled,
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>Speak "friend"!</Trans>;
       <Trans id="custom-id">Speak "friend"!</Trans>;
     "#
);

lingui_test!(
    html_attributes_are_handled,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>
          <Text>This should work &nbsp;</Text>
        </Trans>;
     "#
);

lingui_test!(
    use_decoded_html_entities,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>&amp;</Trans>
     "#
);

lingui_test!(
    elements_inside_expression_container,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>{<span>Component inside expression container</span>}</Trans>;
     "#
);

lingui_test!(
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

lingui_test!(
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

lingui_test!(
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

lingui_test!(
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

lingui_test!(
    use_js_macro_in_jsx_attrs,
    r#"
        import { t } from '@lingui/core/macro';
        import { Trans } from '@lingui/react/macro';
        <Trans>Read <a href="/more" title={t`Full content of ${articleName}`}>more</a></Trans>
     "#
);

lingui_test!(
    use_js_plural_in_jsx_attrs,
    r#"
        import { plural } from '@lingui/core/macro';
        <a href="/about" title={plural(count, {
          one: "\# book",
          other: "\# books"
        })}>About</a>
     "#
);

lingui_test!(
    ignore_jsx_empty_expression,
    r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>Hello {/* and I cannot stress this enough */} World</Trans>;
     "#
);

lingui_test!(
    production,
    production_only_essential_props_are_kept,
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

lingui_test!(
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
lingui_test!(
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

lingui_test!(
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
