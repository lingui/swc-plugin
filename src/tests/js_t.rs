use crate::{to};

to!(
    js_should_not_touch_code_if_no_macro_import,
    // input
     r#"
     t`Refresh inbox`;
     "#,
    // output after transform
    r#"
    t`Refresh inbox`;
    "#
);

to!(
    js_should_not_touch_not_related_tagget_tpls,
    // input
     r#"
     import { t } from "@lingui/macro";

     b`Refresh inbox`;
     b(i18n)`Refresh inbox`;
     "#,
    // output after transform
    r#"
    b`Refresh inbox`;
    b(i18n)`Refresh inbox`;
    "#
);

to!(
    js_substitution_in_tpl_literal,
    // input
     r#"
     import { t } from "@lingui/macro";

     t`Refresh inbox`
     t`Refresh ${foo} inbox ${bar}`
     t`Refresh ${foo.bar} inbox ${bar}`
     t`Refresh ${expr()}`
     "#,
    // output after transform
    r#"
    import { i18n } from "@lingui/core";

    i18n._("Refresh inbox")
    i18n._("Refresh {foo} inbox {bar}", {foo: foo, bar: bar})
    i18n._("Refresh {0} inbox {bar}", {bar: bar, 0: foo.bar})
    i18n._("Refresh {0}", {0: expr()})
    "#
);

to!(
    js_dedup_values_in_tpl_literal,
    // input
     r#"
     import { t } from "@lingui/macro";
     t`Refresh ${foo} inbox ${foo}`
     "#,
    // output after transform
    r#"
    import { i18n } from "@lingui/core";
    i18n._("Refresh {foo} inbox {foo}", {foo: foo})
    "#
);

to!(
    js_custom_i18n_passed,
    // input
     r#"
     import { t } from "@lingui/macro";
     import { custom_i18n } from "./i18n";

     t(custom_i18n)`Refresh inbox`
     t(custom_i18n)`Refresh ${foo} inbox ${bar}`
     t(custom_i18n)`Refresh ${foo.bar} inbox ${bar}`
     t(custom_i18n)`Refresh ${expr()}`
     "#,
    // output after transform
    r#"
    import { custom_i18n } from "./i18n";

    custom_i18n._("Refresh inbox")
    custom_i18n._("Refresh {foo} inbox {bar}", {foo: foo, bar: bar})
    custom_i18n._("Refresh {0} inbox {bar}", {bar: bar, 0: foo.bar})
    custom_i18n._("Refresh {0}", {0: expr()})
    "#
);


to!(
    js_newlines_are_preserved,
    r#"
       import { t } from '@lingui/macro';
         t`Multiline
           string`;
    "#,
     r#"
        import { i18n } from "@lingui/core";
        i18n._("Multiline\nstring");
     "#
);

to!(
    js_support_message_descriptor_in_t_fn,
    r#"
        import { t } from '@lingui/macro'
        const msg = t({ message: `Hello ${name}`, id: 'msgId', comment: 'description for translators'  })
    "#,
     r#"
         import { i18n } from "@lingui/core";
         const msg = i18n._({
          message: "Hello {name}",
          values: {
            name: name,
          },
          id: 'msgId',
          comment: 'description for translators',
         });
     "#
);

to!(
    production,
    js_should_kept_only_essential_props,
    r#"
        import { t } from '@lingui/macro'
        const msg = t({
            message: `Hello ${name}`,
            id: 'msgId',
            comment: 'description for translators',
            context: 'My Context',
        })
    "#,
     r#"
         import { i18n } from "@lingui/core";
         const msg = i18n._({
          values: {
            name: name,
          },
          id: 'msgId',
          context: 'My Context',
         });
     "#
);

to!(
    js_support_template_strings_in_t_macro_message_with_custom_i18n_instance,
    r#"
    import { t } from '@lingui/macro'
    import { i18n_custom } from './lingui'
    const msg = t(i18n_custom)({ message: `Hello ${name}` })
    "#,
    r#"
    import { i18n_custom } from './lingui';
    const msg = i18n_custom._({
      id: "Hello {name}",
      values: {
        name: name,
      },
    });
    "#
);

to!(
    support_id_and_comment_in_t_macro_as_call_expression,
    r#"
        import { t } from '@lingui/macro'
        const msg = t({ id: 'msgId', comment: 'description for translators', message: plural(val, { one: '...', other: '...' }) })
    "#,
    r#"
    import { i18n } from "@lingui/core";
    const msg = i18n._({
      id: 'msgId',
      comment: 'description for translators',
      message: "{val, plural, one {...} other {...}}",
      values: {
        val: val,
      },
    });
    "#
);

to!(
    support_id_in_template_literal,
    r#"
        import { t } from '@lingui/macro'
        const msg = t({ id: `msgId` })
    "#,
    r#"
        import { i18n } from "@lingui/core";
        const msg = i18n._({
          id: `msgId`
        });
    "#
);
