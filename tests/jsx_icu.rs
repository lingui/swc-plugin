use lingui_macro_plugin::LinguiOptions;

#[macro_use]
mod common;

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
     "#
);

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
     "#
);

to!(
    production_only_essential_props_are_kept,
    LinguiOptions {
        strip_non_essential_fields: true,
        ..Default::default()
    },
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
     "#
);

to!(
    multiple_new_lines_with_nbsp_endind,
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  Line ending in non-breaking space.&nbsp;
  <strong>text in element</strong>
</Trans>;
     "#
);
