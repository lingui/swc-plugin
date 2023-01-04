use crate::{to};

to!(
    jsx_simple_jsx,
     r#"
       import { Trans } from "@lingui/macro";
       const exp1 = <Custom>Refresh inbox</Custom>;
       const exp2 = <Trans>Refresh inbox</Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";

       const exp1 = <Custom>Refresh inbox</Custom>;
       const exp2 = <Trans id={"Refresh inbox"} />
    "#
);

to!(
    jsx_preserve_id_and_render_in_trans,
     r#"
       import { Trans } from "@lingui/macro";
       const exp2 = <Trans id="custom.id" render={(v) => v}>Refresh inbox</Trans>;
     "#,
    r#"
       import { Trans } from "@lingui/react";
       const exp2 = <Trans message={"Refresh inbox"} id="custom.id" render={(v) => v} />
    "#
);

to!(
    jsx_interpolation,
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
          Hello {foo} and {foo}
        </Trans>
     "#,
    r#"
       import { Trans } from "@lingui/react";
       <Trans id={"Hello {foo} and {foo}"} values={{
          foo: foo,
        }}/>;
    "#
);

to!(
    jsx_icu,
     r#"
      import { Plural } from "@lingui/macro";

      <Plural
       value={count}
       one="Message"
       other="Messages"
      />
     "#,

    r#"
       import { Trans } from "@lingui/react";

       <Trans
           id={"{count, plural, one {Message} other {Messages}}"}
           values={{ count: count }}
        />
    "#
);

to!(
    jsx_icu_explicit_id,
     r#"
       import { Plural } from "@lingui/macro";

      <Plural
       id="plural.id"
       value={count}
       one="Message"
       other="Messages"
      />
     "#,

    r#"
       import { Trans } from "@lingui/react";

       <Trans
           message={"{count, plural, one {Message} other {Messages}}"}
           values={{ count: count }}
           id="plural.id"
        />
    "#
);

to!(
    jsx_icu_nested,
     r#"
       import { Plural } from "@lingui/macro";

       <Trans>
       You have{" "}
          <Plural
           value={count}
           one="Message"
           other="Messages"
          />
      </Trans>
     "#,

    r#"
       import { Trans } from "@lingui/react";

       <Trans
           id={"You have {count, plural, one {Message} other {Messages}}"}
           values={{ count: count }}
        />
    "#
);

to!(
    jsx_trans_inside_plural,
     r#"
       import { Trans, Plural } from '@lingui/macro';
        <Plural
          value={count}
          one={
            <Trans>
              <strong>#</strong> slot added
            </Trans>
          }
          other={
            <Trans>
              <strong>#</strong> slots added
            </Trans>
          }
        />;
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans id={
          "{count, plural, one {<0>#</0> slot added} other {<1>#</1> slots added}}"
        }
        values={{
          count: count
        }} components={{
          0: <strong />,
          1: <strong />
        }} />;

    "#
);

to!(
    jsx_multivelel_nesting,
     r#"
import { Trans, Plural } from '@lingui/macro';

<Plural
  value={count}
  one={
    <Trans>
      <Plural
        value={count2}
        one={
          <Trans>
            second level one
          </Trans>
        }
        other={
          <Trans>
            second level other
          </Trans>
        }
      />

      <strong>#</strong> slot added
    </Trans>
  }
  other={
    <Trans>
      <strong>#</strong> slots added
    </Trans>
  }
/>;
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans id={
          "{count, plural, one {{count2, plural, one { second level one} other { second level other}}<0>#</0> slot added} other {<1>#</1> slots added}}"
        }
        values={{
          count: count,
          count2: count2
        }} components={{
          0: <strong />,
          1: <strong />
        }} />;
    "#
);