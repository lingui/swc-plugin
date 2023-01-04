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
    js_icu_functions,
     r#"
    import { plural, select, selectOrdinal } from "@lingui/macro";
    const messagePlural = plural(count, {
       one: '# Book',
       other: '# Books'
    })
    const messageSelect = select(gender, {
       male: 'he',
       female: 'she',
       other: 'they'
    })
    const messageSelectOrdinal = selectOrdinal(count, {
       one: '#st',
       two: '#nd',
       few: '#rd',
       other: '#th',
    })
     "#,
    r#"
    import { i18n } from "@lingui/core";
    const messagePlural = i18n._("{count, plural, one {# Book} other {# Books}}", {
      count: count
    });
    const messageSelect = i18n._("{gender, select, male {he} female {she} other {they}}", {
      gender: gender
    });
    const messageSelectOrdinal = i18n._("{count, selectOrdinal, one {#st} two {#nd} few {#rd} other {#th}}", {
      count: count
    });
    "#
);

to!(
    js_should_not_touch_non_lungui_fns,
     r#"
    import { plural } from "@lingui/macro";
    const messagePlural = customName(count, {
       one: '# Book',
       other: '# Books'
    })
     "#,
    r#"
   const messagePlural = customName(count, {
       one: '# Book',
       other: '# Books'
    })
    "#
);


to!(
    js_plural_with_placeholders,
     r#"
       import { plural } from "@lingui/macro";

       const message = plural(count, {
           one: `${name} has # friend`,
           other: `${name} has # friends`
        })
     "#,
    r#"
    import { i18n } from "@lingui/core";
    const message = i18n._("{count, plural, one {{name} has # friend} other {{name} has # friends}}", {
      count: count,
      name: name,
    })
    "#
);

to!(
    js_dedup_values_in_icu,
     r#"
       import { plural } from "@lingui/macro";

       const message = plural(count, {
           one: `${name} has ${count} friend`,
           other: `${name} has {count} friends`
        })
     "#,
    r#"
    import { i18n } from "@lingui/core";

    const message = i18n._("{count, plural, one {{name} has {count} friend} other {{name} has {count} friends}}", {
      count: count,
      name: name,
    })
    "#
);