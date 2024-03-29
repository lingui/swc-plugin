use crate::{to};

to!(
    jsx_icu,
     r#"
import { Plural } from "@lingui/react/macro";

const ex1 = <Plural
 value={count}
 one="Message"
 other="Messages"
/>

const ex2 = <div><Plural
value={count}
one="Message"
other="Messages"
/></div>
     "#,

    r#"
import { Trans } from "@lingui/react";

const ex1 = <Trans message={"{count, plural, one {Message} other {Messages}}"} id={"V4EO9s"} values={{
    count: count
}}/>;
const ex2 = <div ><Trans message={"{count, plural, one {Message} other {Messages}}"} id={"V4EO9s"} values={{
    count: count
}}/></div>;

    "#
);

to!(
    jsx_icu_explicit_id,
     r#"
       import { Plural } from "@lingui/react/macro";

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
    jsx_plural_preserve_reserved_attrs,
     r#"
       import { Plural } from "@lingui/react/macro";

      <Plural
       comment="Translators Comment"
       context="Message Context"
       render={(v) => v}
       value={count}
       one="..."
       other="..."
      />
     "#,

    r#"
       import { Trans } from "@lingui/react";

       <Trans
           message={"{count, plural, one {...} other {...}}"}
           id={"8I55rI"}
           values={{ count: count }}
           render={(v) => v}
        />
    "#
);

to!(
    jsx_icu_nested,
     r#"
       import { Plural, Trans } from "@lingui/react/macro";

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
           message={"You have {count, plural, one {Message} other {Messages}}"}
           id={"dzhU0t"}
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
        <Trans message={"{count, plural, one {<0>#</0> slot added} other {<1>#</1> slots added}}"} id={"X8eyr1"}
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
        <Trans
        message={"{count, plural, one {{count2, plural, one {second level one} other {second level other}}<0>#</0> slot added} other {<1>#</1> slots added}}"}
        id={"bDgQmM"}
        values={{
          count: count,
          count2: count2
        }} components={{
          0: <strong />,
          1: <strong />
        }} />;
    "#
);

to!(
    jsx_plural_with_offset_and_exact_matches,
     r#"
       import { Plural } from "@lingui/react/macro";

        <Plural
          value={count}
          offset="1"
          _0="Zero items"
          other={<a href="/more">A lot of them</a>}
        />;
     "#,

    r#"
       import { Trans } from "@lingui/react";
        <Trans message={"{count, plural, offset:1 =0 {Zero items} other {<0>A lot of them</0>}}"}
         id={"ZFknU1"}
         values={{
          count: count
        }} components={{
          0: <a href="/more" />
        }} />;
    "#
);

to!(
    jsx_icu_with_template_literal,
     r#"
       import { Plural } from "@lingui/react/macro";

        <Plural
          value={count}
          one={`${count} items`}
          other="..."
        />;
     "#,

    r#"
       import { Trans } from "@lingui/react";
        <Trans
        message={"{count, plural, one {{count} items} other {...}}"}
        id={"+hE+5/"}
         values={{
          count: count
        }}
        />;
    "#
);

to!(
    jsx_select_simple,
     r#"
        import { Select } from '@lingui/macro';
        <Select
          value={count}
          _male="He"
          _female={`She`}
          other={<strong>Other</strong>}
        />;
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"{count, select, male {He} female {She} other {<0>Other</0>}}"} id={"Imwef9"} values={{
          count: count
        }} components={{
          0: <strong />
        }} />;
    "#
);

to!(
    jsx_select_with_expressions_in_cases,
     r#"
        import { Select } from '@lingui/macro';
        <Select
          value={count}
          _male={variable}
          _third={foo.bar}
          _female={`She`}
          other={<strong>Other</strong>}
        />;
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"{count, select, male {{variable}} third {{0}} female {She} other {<0>Other</0>}}"} id={"/7RSeH"} values={{
          count: count,
          variable: variable,
          0: foo.bar
        }} components={{
          0: <strong />
        }} />;
    "#
);

to!(
    jsx_select_with_reserved_attrs,
     r#"
        import { Select } from '@lingui/macro';
        <Select
          comment="Translators Comment"
          context="Message Context"
          render={(v) => v}

          value={count}
          _male="He"
          _female={`She`}
          other={<strong>Other</strong>}
        />;
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans message={"{count, select, male {He} female {She} other {<0>Other</0>}}"} id={"4jX4Bx"} values={{
          count: count
        }} components={{
          0: <strong />
        }}
          render={(v) => v}
           />;
    "#
);

// // https://github.com/lingui/js-lingui/issues/1324
// to!(
//     jsx_select_options_should_work_without_underscore,
//      r#"
//         import { Select } from '@lingui/macro';
//         <Select
//           value={count}
//           male="He"
//           female={`She`}
//           other={<strong>Other</strong>}
//         />;
//      "#,
//
//     r#"
//         import { Trans } from "@lingui/react";
//         <Trans id={"{count, select, male {He} female {She} other {<0>Other</0>}}"} values={{
//           count: count
//         }} components={{
//           0: <strong />
//         }}
//            />;
//     "#
// );

to!(
    jsx_select_ordinal_with_offset_and_exact_matches,
     r#"
       import { SelectOrdinal } from "@lingui/react/macro";

        <SelectOrdinal
          value={count}
          offset="1"
          _0=" #st"
          one=" #nd"
          other=' #rd'
        />;
     "#,

    r#"
       import { Trans } from "@lingui/react";
        <Trans message={"{count, selectordinal, offset:1 =0 { #st} one { #nd} other { #rd}}"} id={"cp8FR4"} values={{
            count: count
        }}/>;
    "#
);

to!(
    production,
    production_only_essential_props_are_kept,
     r#"
        import { Plural } from '@lingui/macro';

        <Plural
          id="custom.id"
          context="My Context"
          comment="This is for translators"
          render="render"
          i18n="i18n"
          value={count}
          offset="1"
          _0="Zero items"
          other={<a href="/more">A lot of them</a>}
          />
     "#,

    r#"
        import { Trans } from "@lingui/react";
        <Trans
        values={{count: count}}
        components={{0: <a href="/more"/>}}
        id="custom.id"
        render="render"
        i18n="i18n" />;
    "#
);

