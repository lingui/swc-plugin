use lingui_macro_plugin::{DescriptorFields, LinguiOptions};

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
    js_id_only_should_keep_only_id,
    LinguiOptions {
        descriptor_fields: DescriptorFields::IdOnly,
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
    js_message_should_keep_message_and_context,
    LinguiOptions {
        descriptor_fields: DescriptorFields::Message,
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

to!(
    js_should_use_v5_generate_id_with_parameter,
    LinguiOptions {
        use_lingui_v5_id_generation: true,
        ..Default::default()
    },
    r#"
     import { t } from '@lingui/core/macro'
     t({
       message: "Hello World",
       context: "my context"
     });
     "#
);

to!(
    js_t_with_directive_context_block_comment,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="my context" */
        const msg = t`Hello`
    "#
);

to!(
    js_t_with_directive_comment_line_comment,
    r#"
        import { t } from '@lingui/core/macro';
        // lingui-set comment="translator note"
        const msg = t`Hello`
    "#
);

to!(
    js_t_with_directive_context_and_comment,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="ctx" comment="cmt" */
        const msg = t`Hello`
    "#
);

to!(
    js_t_directive_applies_to_multiple_subsequent_macros,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="shared" */
        const msg1 = t`Hello`
        const msg2 = t`World`
    "#
);

to!(
    js_t_closest_directive_takes_precedence,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="first" */
        const msg1 = t`Hello`
        /* lingui-set context="second" */
        const msg2 = t`World`
    "#
);

to!(
    js_t_directives_merge_with_preceding_ones,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="ctx" comment="cmt" */
        const msg1 = t`Hello`
        /* lingui-set context="new ctx" */
        const msg2 = t`World`
    "#
);

to!(
    js_t_reset_clears_all_inherited_values,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="first" comment="second" idPrefix="prefix." */
        const msg1 = t`Hello`
        /* lingui-reset */
        const msg2 = t`World`
    "#
);

to!(
    js_t_reset_combined_with_new_values,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="first" comment="second" */
        const msg1 = t`Hello`
        /* lingui-reset context="fresh" */
        const msg2 = t`World`
    "#
);

to!(
    js_t_empty_param_value_clears_single_param,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="first" comment="second" */
        const msg1 = t`Hello`
        /* lingui-set context="" */
        const msg2 = t`World`
    "#
);

to!(
    js_t_explicit_comment_overrides_directive,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set comment="directive cmt" */
        const msg = t({ message: "Hello", comment: "explicit cmt" })
    "#
);

to!(
    js_t_id_prefix_with_explicit_id,
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set idPrefix="module." */
        const msg = t({ id: "greeting", message: "Hello" })
    "#
);

to!(
    js_t_with_id_prefix_leader,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set idPrefix="module" comment="cmt" */
        const msg = t({ id: "unprefixed", message: "Welcome" })
        const msg2 = t({ id: ".my.id", message: "Welcome" })
    "#
);

to!(
    js_t_id_prefix_leader_with_dynamic_id,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set idPrefix="module" */
        const dynId = "dynamic";
        const msg = t({ id: dynId, message: "Welcome" })
    "#
);

to!(
    js_t_id_only_with_directive_context_uses_context_for_hash,
    LinguiOptions {
        descriptor_fields: DescriptorFields::IdOnly,
        ..Default::default()
    },
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context="my context" */
        const msg = t`Hello`
    "#
);
