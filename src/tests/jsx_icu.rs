use crate::{to};

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

to!(
    jsx_icu_with_offset_and_exact_matches,
     r#"
       import { Plural } from "@lingui/macro";

        <Plural
          value={count}
          offset="1"
          _0="Zero items"
          other={<a href="/more">A lot of them</a>}
        />;
     "#,

    r#"
       import { Trans } from "@lingui/react";
        <Trans id={
          "{count, plural, offset:1 =0 {Zero items} other {<0>A lot of them</0>}}"
         }
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
       import { Plural } from "@lingui/macro";

        <Plural
          value={count}
          one={`${count} items`}
          other="..."
        />;
     "#,

    r#"
       import { Trans } from "@lingui/react";
        <Trans id={
          "{count, plural, one {{count} items} other {...}}"
         }
         values={{
          count: count
        }}
        />;
    "#
);
