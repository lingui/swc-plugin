use crate::{to};

to!(
    jsx_simple_jsx,
     r#"
       import { Trans } from "@lingui/macro";
       const exp1 = <Custom>Refresh inbox</Custom>;
       const exp2 = <Trans>Refresh inbox</Trans>;
       const exp3 = <div><Trans>Refresh inbox</Trans></div>;
     "#,
    r#"
       import { Trans } from "@lingui/react";

       const exp1 = <Custom>Refresh inbox</Custom>;
       const exp2 = <Trans id={"Refresh inbox"} />;
       const exp3 = <div><Trans id={"Refresh inbox"} /></div>;
    "#
);


to!(
    jsx_with_custom_id,
     r#"
       import { Trans } from "@lingui/macro";
       const exp2 = <Trans id="custom.id">Refresh inbox</Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";
       const exp2 = <Trans message={"Refresh inbox"} id="custom.id" />
    "#
);

to!(
    jsx_preserve_reserved_attrs,
     r#"
       import { Trans } from "@lingui/macro";
       const exp2 = <Trans comment="Translators Comment" context="Message Context" i18n="i18n" render={(v) => v}>Refresh inbox</Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";
       const exp2 = <Trans id={"Refresh inbox"} comment="Translators Comment" context="Message Context" i18n="i18n" render={(v) => v} />
    "#
);

to!(
    jsx_expressions_are_converted_to_positional_arguments,
     r#"
       import { Trans } from "@lingui/macro";
       <Trans>
          Property {props.name},
          function {random()},
          array {array[index]},
          constant {42},
          object {new Date()},
          everything {props.messages[index].value()}
        </Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";
       <Trans id={"Property {0}, function {1}, array {2}, constant {3}, object {4}, everything {5}"} values={{
          0: props.name,
          1: random(),
          2: array[index],
          3: 42,
          4: new Date(),
          5: props.messages[index].value()
        }} />;
    "#
);

to!(
    jsx_components_interpolation,
     r#"
       import { Trans } from "@lingui/macro";
       <Trans>
          Hello <strong>World!</strong><br />
          <p>
            My name is <a href="/about">{" "}
            <em>{name}</em></a>
          </p>
        </Trans>
     "#,
    r#"
    import { Trans } from "@lingui/react";
   <Trans id={"Hello <0>World!</0><1/><2>My name is <3> <4>{name}</4></3></2>"} values={{
      name: name,
    }} components={{
      0: <strong />,
      1: <br />,
      2: <p />,
      3: <a href="/about" />,
      4: <em />
    }} />;
    "#
);

to!(
    jsx_values_dedup,
     r#"
       import { Trans } from "@lingui/macro";
       <Trans>
          Hello {foo} and {foo}{" "}
          {bar}
        </Trans>
     "#,
    r#"
       import { Trans } from "@lingui/react";
       <Trans id={"Hello {foo} and {foo} {bar}"} values={{
          foo: foo,
          bar: bar,
        }}/>;
    "#
);

to!(
    jsx_template_literal_in_children,
     r#"
       import { Trans } from "@lingui/macro";
       <Trans>{`Hello ${foo} and ${bar}`}</Trans>
     "#,
    r#"
       import { Trans } from "@lingui/react";
       <Trans id={"Hello {foo} and {bar}"} values={{
          foo: foo,
          bar: bar,
        }}/>;
    "#
);

to!(
    quoted_jsx_attributes_are_handled,
     r#"
       import { Trans } from '@lingui/macro';
       <Trans>Speak "friend"!</Trans>;
       <Trans id="custom-id">Speak "friend"!</Trans>;
     "#,

    r#"
      import { Trans } from "@lingui/react";
      <Trans id={'Speak "friend"!'} />;
      <Trans message={'Speak "friend"!'} id="custom-id" />;
    "#
);

to!(
    html_attributes_are_handled,
     r#"
        import { Trans } from '@lingui/macro';
        <Trans>
          <Text>This should work &nbsp;</Text>
        </Trans>;
     "#,

    r#"
     import { Trans } from "@lingui/react";
     <Trans id={"<0>This should work \xa0</0>"}
        components={{
          0: <Text />,
        }}
     />;
    "#
);

to!(
    use_decoded_html_entities,
     r#"
        import { Trans } from "@lingui/macro";
        <Trans>&amp;</Trans>
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"&"} />;
    "#
);


to!(
    elements_inside_expression_container,
     r#"
        import { Trans } from '@lingui/macro';
        <Trans>{<span>Component inside expression container</span>}</Trans>;
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"<0>Component inside expression container</0>"} components={{
          0: <span />
        }} />;
    "#
);

to!(
    elements_without_children,
     r#"
        import { Trans } from '@lingui/macro';
        <Trans>{<br />}</Trans>;
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"<0/>"} components={{
          0: <br />
        }} />;
    "#
);

// it's better to throw panic here
// to!(
//     jsx_spread_child_is_noop,
//      r#"
//         import { Trans } from '@lingui/macro';
//         <Trans>{...spread}</Trans>
//      "#,
//     r#"
//         import { Trans } from "@lingui/react";
//         <Trans>{...spread}</Trans>
//     "#
// );

to!(
    strip_whitespace_around_arguments,
     r#"
        import { Trans } from "@lingui/macro";
        <Trans>
          Strip whitespace around arguments: '
          {name}
          '
        </Trans>
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"Strip whitespace around arguments: '{name}'"} values={{
          name: name
        }} />;
    "#
);

to!(
    strip_whitespace_around_tags_but_keep_forced_spaces,
     r#"
        import { Trans } from "@lingui/macro";
        <Trans>
          Strip whitespace around tags, but keep{" "}
          <strong>forced spaces</strong>
          !
        </Trans>
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"Strip whitespace around tags, but keep <0>forced spaces</0>!"} components={{
          0: <strong />
        }} />;
    "#
);

to!(
    keep_forced_newlines,
     r#"
        import { Trans } from "@lingui/macro";
        <Trans>
          Keep forced{"\\n"}
          newlines!
        </Trans>
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"Keep forced\n newlines!"} />;
    "#
);

to!(
    keep_multiple_forced_newlines,
     r#"
        import { Trans } from "@lingui/macro";
        <Trans>
          Keep multiple{"\\n"}
          forced{"\\n"}
          newlines!
        </Trans>
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"Keep multiple\n forced\n newlines!"} />;
    "#
);

to!(
    use_js_macro_in_jsx_attrs,
     r#"
        import { t, Trans } from '@lingui/macro';
        <Trans>Read <a href="/more" title={t`Full content of ${articleName}`}>more</a></Trans>
     "#,
    r#"
        import { Trans } from "@lingui/react";
        import { i18n } from "@lingui/core";
        <Trans id={"Read <0>more</0>"} components={{
            0: <a href="/more" title={i18n._({
                id: "qzc3IN",
                message: "Full content of {articleName}",
                values: {
                    articleName: articleName
                }
            })}/>
        }}/>;
    "#
);

to!(
    use_js_plural_in_jsx_attrs,
     r#"
        import { plural } from '@lingui/macro';
        <a href="/about" title={plural(count, {
          one: "\# book",
          other: "\# books"
        })}>About</a>
     "#,

    r#"
        import { i18n } from "@lingui/core";
        <a href="/about" title={i18n._({
          id: "esnaQO",
          message: "{count, plural, one {# book} other {# books}}",
          values: {
              count: count
          }
      })}>About</a>;

    "#
);

to!(
    ignore_jsx_empty_expression,
     r#"
        import { Trans } from '@lingui/macro';
        <Trans>Hello {/* and I cannot stress this enough */} World</Trans>;
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans id={"Hello  World"} />;
    "#
);

to!(
    production,
    production_only_essential_props_are_kept,
     r#"
        import { Trans } from '@lingui/macro';
        <Trans
        id="msg.hello"
        render="render"
        i18n="i18n"
        context="My Context"
        comment="Hello World">Hello <strong>{name}</strong></Trans>
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans
            values={{name: name}}
            components={{0: <strong />}}
            id="msg.hello"
            render="render"
            i18n="i18n"
            context="My Context"
        />;
    "#
);

//   {
//     name: "production - import_type_doesn't_interference_on_normal_import",
//     production: true,
//     useTypescriptPreset: true,
//     input: `
//         import { withI18nProps } from '@lingui/react'
//         import { Trans } from '@lingui/macro';
//         <Trans id="msg.hello" comment="Hello World">Hello World</Trans>
//       `,
//     expected: `
//         import { withI18nProps, Trans } from "@lingui/react";
//         <Trans id="msg.hello" />;
//       `,
//   },

