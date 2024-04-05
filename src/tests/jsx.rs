use crate::{to};

to!(
    jsx_simple_jsx,
     r#"
import { Trans } from "@lingui/react/macro";
const exp1 = <Custom>Refresh inbox</Custom>;
const exp2 = <Trans>Refresh inbox</Trans>;
const exp3 = <div><Trans>Refresh inbox</Trans></div>;
     "#,
    r#"
import { Trans } from "@lingui/react";
const exp1 = <Custom >Refresh inbox</Custom>;
const exp2 = <Trans message={"Refresh inbox"} id={"EsCV2T"}/>;
const exp3 = <div><Trans message={"Refresh inbox"} id={"EsCV2T"}/></div>;
    "#
);

to!(
    jsx_should_suppor_legacy_import,
     r#"
import { Trans } from "@lingui/macro";
const exp2 = <Trans>Refresh inbox</Trans>;
     "#,
    r#"
import { Trans } from "@lingui/react";
const exp2 = <Trans message={"Refresh inbox"} id={"EsCV2T"}/>;
    "#
);

to!(
    jsx_with_custom_id,
     r#"
       import { Trans } from "@lingui/react/macro";
       const exp2 = <Trans id="custom.id">Refresh inbox</Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";
       const exp2 = <Trans message={"Refresh inbox"} id="custom.id" />
    "#
);

to!(
    jsx_with_context,
     r#"
       import { Trans } from "@lingui/react/macro";
       const exp1 = <Trans>Refresh inbox</Trans>;
       const exp2 = <Trans context="My Context">Refresh inbox</Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";
       const exp1 = <Trans message={"Refresh inbox"} id={"EsCV2T"}/>;
       const exp2 = <Trans message={"Refresh inbox"} id={"BAyVRj"}/>;
    "#
);

to!(
    jsx_preserve_reserved_attrs,
     r#"
       import { Trans } from "@lingui/react/macro";
       const exp2 = <Trans comment="Translators Comment" context="Message Context" i18n="i18n" render={(v) => v}>Refresh inbox</Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";
       const exp2 = <Trans message={"Refresh inbox"} id={"6J8UtY"} i18n="i18n" render={(v) => v} />
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
     "#,
    r#"
       import { Trans } from "@lingui/react";
       <Trans
         message={"Property {0}, function {1}, array {2}, constant {3}, object {4}, everything {5}"}
         id={"HjKDmx"}
         values={{
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
       import { Trans } from "@lingui/react/macro";
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
   <Trans
   message={"Hello <0>World!</0><1/><2>My name is <3> <4>{name}</4></3></2>"}
   id={"k9gsHO"}
   values={{
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
       import { Trans } from "@lingui/react/macro";
       <Trans>
          Hello {foo} and {foo}{" "}
          {bar}
        </Trans>
     "#,
    r#"
       import { Trans } from "@lingui/react";
       <Trans message={"Hello {foo} and {foo} {bar}"} id={"LyhmWJ"}
        values={{
          foo: foo,
          bar: bar,
        }}/>;
    "#
);

to!(
    jsx_template_literal_in_children,
     r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>{`Hello ${foo} and ${bar}`}</Trans>
     "#,
    r#"
       import { Trans } from "@lingui/react";
       <Trans message={"Hello {foo} and {bar}"} id={"pB1yir"} values={{
          foo: foo,
          bar: bar,
        }}/>;
    "#
);

to!(
    quoted_jsx_attributes_are_handled,
     r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>Speak "friend"!</Trans>;
       <Trans id="custom-id">Speak "friend"!</Trans>;
     "#,

    r#"
      import { Trans } from "@lingui/react";
      <Trans message={'Speak "friend"!'} id={"NWmRwM"}/>;
      <Trans message={'Speak "friend"!'} id="custom-id" />;
    "#
);

to!(
    html_attributes_are_handled,
     r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>
          <Text>This should work &nbsp;</Text>
        </Trans>;
     "#,

    r#"
     import { Trans } from "@lingui/react";
     <Trans message={"<0>This should work  </0>"} id={"K/1Xpr"}
        components={{
          0: <Text />,
        }}
     />;
    "#
);

to!(
    use_decoded_html_entities,
     r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>&amp;</Trans>
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"&"} id={"EwTON7"}/>;
    "#
);


to!(
    elements_inside_expression_container,
     r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>{<span>Component inside expression container</span>}</Trans>;
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans
          message={"<0>Component inside expression container</0>"}
          id={"1cZQQW"}
         components={{
          0: <span />
        }} />;
    "#
);

to!(
    elements_without_children,
     r#"
        import { Trans } from "@lingui/react/macro";
        <Trans>{<br />}</Trans>;
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"<0/>"} id={"SCJtqt"} components={{
          0: <br />
        }} />;
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
//         import { Trans } from "@lingui/react";
//         <Trans>{...spread}</Trans>
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
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"Strip whitespace around arguments: '{name}'"} id={"tRMgLt"} values={{
          name: name
        }} />;
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
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"Strip whitespace around tags, but keep <0>forced spaces</0>!"} id={"Ud4KOf"} components={{
          0: <strong />
        }} />;
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
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"Keep multiple\nforced\nnewlines!"} id={"9xE5pD"}/>;
    "#
);

to!(
    use_js_macro_in_jsx_attrs,
     r#"
        import { t } from '@lingui/core/macro';
        import { Trans } from '@lingui/react/macro';
        <Trans>Read <a href="/more" title={t`Full content of ${articleName}`}>more</a></Trans>
     "#,
    r#"
        import { Trans } from "@lingui/react";
        import { i18n } from "@lingui/core";
        <Trans  message={"Read <0>more</0>"} id={"QZyANg"} components={{
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
        import { plural } from '@lingui/core/macro';
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
        import { Trans } from "@lingui/react/macro";
        <Trans>Hello {/* and I cannot stress this enough */} World</Trans>;
     "#,
    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"Hello  World"} id={"i0M2R8"} />;
    "#
);

to!(
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
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans
            values={{name: name}}
            components={{0: <strong />}}
            id="msg.hello"
            render="render"
            i18n="i18n"
        />;
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
     "#,

    r#"
      import { Trans } from "@lingui/react";

      <Trans
        message={
          "Wonderful framework <0>Next.js</0> say hi. And <1>Next.js</1> say hi."
        }
        id={"3YVd0H"}
        components={{
          0: <a href="https://nextjs.org" />,
          1: <a href="https://nextjs.org" />,
        }}
      />;
    "#
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

