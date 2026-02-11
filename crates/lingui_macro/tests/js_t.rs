use lingui_macro::LinguiOptions;

#[macro_use]
mod common;

to!(
    js_should_not_touch_code_if_no_macro_import,
    r#"
     t`Refresh inbox`;
     "#
);

to!(
    js_should_not_touch_not_related_tagget_tpls,
    r#"
     import { t } from "@lingui/core/macro";

     b`Refresh inbox`;
     b(i18n)`Refresh inbox`;
     "#
);

to!(
    js_should_work_with_legacy_import,
    r#"
     import { t } from "@lingui/macro";

    t`Refresh inbox`;
     "#
);

to!(
    js_substitution_in_tpl_literal,
    r#"
     import { t } from "@lingui/core/macro";

     t`Refresh inbox`
     t`Refresh ${foo} inbox ${bar}`
     t`Refresh ${foo.bar} inbox ${bar}`
     t`Refresh ${expr()}`
     "#
);

to!(
    js_dedup_values_in_tpl_literal,
    r#"
     import { t } from "@lingui/core/macro";
     t`Refresh ${foo} inbox ${foo}`
     "#
);

to!(
    js_explicit_labels_in_tpl_literal,
    r#"
   import { t } from "@lingui/core/macro";

   t`Refresh ${{foo}} inbox`
   t`Refresh ${{foo: foo.bar}} inbox`
   t`Refresh ${{foo: expr()}} inbox`
   t`Refresh ${{foo: bar, baz: qux}} inbox`
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
  t`Refresh ${ph({foo: bar, baz: qux})} inbox`
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

to!(
    js_custom_i18n_passed,
    r#"
     import { t } from "@lingui/core/macro";
     import { custom_i18n } from "./i18n";

     t(custom_i18n)`Refresh inbox`
     t(custom_i18n)`Refresh ${foo} inbox ${bar}`
     t(custom_i18n)`Refresh ${foo.bar} inbox ${bar}`
     t(custom_i18n)`Refresh ${expr()}`
     t(custom.i18n)`Refresh ${expr()}`
     "#
);

to!(
    js_newlines_are_preserved,
    r#"
       import { t } from '@lingui/core/macro';
         t`Multiline
           string`;
    "#
);

to!(
    js_continuation_character,
    r#"
       import { t } from '@lingui/core/macro';
         t`Multiline\
           string`;
    "#
);

to!(
    unicode_characters_interpreted,
    r#"
       import { t } from '@lingui/core/macro';
       t`Message \u0020`;
       t`Bienvenue\xA0!`
    "#
);

to!(
    js_support_message_descriptor_in_t_fn,
    r#"
        import { t } from '@lingui/core/macro'
        const msg = t({ message: `Hello ${name}`, id: 'msgId', comment: 'description for translators'  })
    "#
);

to!(
    js_t_fn_wrapped_in_call_expr,
    r#"
        import { t } from '@lingui/core/macro'
        const msg = message.error(t({message: "dasd"}))
    "#
);

to!(
    js_should_kept_only_essential_props,
    LinguiOptions {
        strip_non_essential_fields: true,
        ..Default::default()
    },
    r#"
        import { t } from '@lingui/core/macro'
        const msg1 = t`Message`
        const msg2 = t({
            message: `Hello ${name}`,
            id: 'msgId',
            comment: 'description for translators',
            context: 'My Context',
        })
    "#
);

to!(
    js_should_produce_all_fields_without_strip_flag,
    r#"
        import { t } from '@lingui/core/macro'
        const msg2 = t({
            message: `Hello ${name}`,
            id: 'msgId',
            comment: 'description for translators',
            context: 'My Context',
        })
    "#
);

to!(
    js_should_produce_all_fields_when_no_message_set,
    r#"
        import { t } from '@lingui/core/macro'
        const msg2 = t({
            id: 'msgId',
            comment: 'description for translators',
            context: 'My Context',
        })
    "#
);

to!(
    js_support_template_strings_in_t_macro_message_with_custom_i18n_instance,
    r#"
    import { t } from '@lingui/core/macro'
    import { i18n_custom } from './lingui'
    const msg = t(i18n_custom)({ message: `Hello ${name}` })
    "#
);

to!(
    support_id_and_comment_in_t_macro_as_call_expression,
    r#"
        import { t, plural } from '@lingui/core/macro'
        const msg = t({ id: 'msgId', comment: 'description for translators', message: plural(val, { one: '...', other: '...' }) })
    "#
);

to!(
    support_id_in_template_literal,
    r#"
        import { t } from '@lingui/core/macro'
        const msg = t({ id: `msgId` })
    "#
);

to!(
    should_generate_diffrent_id_when_context_provided,
    r#"
        import { t } from '@lingui/core/macro'
        t({ message: 'Ola' })
        t({ message: 'Ola', context: "My Context"})
        t({ message: 'Ola', context: `My Context`})
    "#
);
