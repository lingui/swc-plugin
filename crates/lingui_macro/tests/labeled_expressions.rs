use lingui_macro::LinguiOptions;

#[macro_use]
mod common;

// --- JS: labeled expressions in template literals ---

to!(
    js_explicit_labels_in_tpl_literal,
    r#"
   import { t } from "@lingui/core/macro";

   t`Refresh ${{foo}} inbox`
   t`Refresh ${{foo: foo.bar}} inbox`
   t`Refresh ${{foo: expr()}} inbox`
   t`Refresh ${{}} inbox`
   t`Refresh ${{...spread}} inbox`
   "#
);

to!(
    js_ph_labels_in_tpl_literal,
    r#"
  import { t, ph } from "@lingui/core/macro";

  t`Refresh ${ph({foo})} inbox`
  t`Refresh ${ph({foo: foo.bar})} inbox`
  t`Refresh ${ph({foo: expr()})} inbox`
  t`Refresh ${ph({})} inbox`
  t`Refresh ${ph({...spread})} inbox`
  "#
);

to!(
    js_choice_labels_in_tpl_literal,
    r##"
  import { t, ph, plural, select, selectOrdinal } from "@lingui/core/macro";

  t`We have ${plural({count: getDevelopersCount()}, {one: "# developer", other: "# developers"})}`
  t`${select(gender, {male: "he", female: "she", other: "they"})}`
  t`${selectOrdinal(count, {one: "#st", two: "#nd", few: "#rd", other: "#th"})}`
  "##
);

// --- JSX: labeled expressions in Trans children ---

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

// --- JSX: labeled expressions in Plural/Select value ---

to!(
    jsx_icu_with_labeled_expression_as_value,
    r#"
 import { Plural } from '@lingui/react/macro';
        <Plural
          value={{count: getCount()}}
          one={"oneText"}
          other={<a href="/more">A lot of them</a>}
        />;
     "#
);

to!(
    jsx_icu_with_labeled_expression_as_value_with_ph,
    r#"
import { Plural } from '@lingui/react/macro';
        import { ph } from '@lingui/core/macro';
        <Plural
          value={ph({count: getCount()})}
          one={"oneText"}
          other={<a href="/more">A lot of them</a>}
        />
     "#
);

// --- Error cases ---

to_panic!(
    js_ph_with_non_object_arg,
    LinguiOptions::default(),
    r#"
  import { t, ph } from "@lingui/core/macro";
  t`Refresh ${ph(variable)} inbox`
  "#
);

to_panic!(
    js_ph_labeled_expression_multiple_props,
    LinguiOptions::default(),
    r#"
  import { t, ph } from "@lingui/core/macro";
  t`Refresh ${ph({foo: bar, baz: qux})} inbox`
  "#
);

to_panic!(
    js_labeled_expression_multiple_props,
    LinguiOptions::default(),
    r#"
  import { t } from "@lingui/core/macro";
  t`Refresh ${{foo: bar, baz: qux}} inbox`
  "#
);

to_panic!(
    jsx_ph_with_non_object_arg,
    LinguiOptions::default(),
    r#"
       import { Trans, ph } from "@lingui/react/macro";
       <Trans>Refresh {ph(variable)} inbox</Trans>;
     "#
);

to_panic!(
    jsx_labeled_expression_multiple_props,
    LinguiOptions::default(),
    r#"
       import { Trans } from "@lingui/react/macro";
       <Trans>Refresh {{foo: bar, baz: qux}} inbox</Trans>;
     "#
);

to_panic!(
    jsx_ph_labeled_expression_multiple_props,
    LinguiOptions::default(),
    r#"
       import { Trans, ph } from "@lingui/react/macro";
       <Trans>Refresh {ph({foo: bar, baz: qux})} inbox</Trans>;
     "#
);
