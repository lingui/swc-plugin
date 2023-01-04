use crate::{to};

to!(
    js_icu_macro,
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
    const messageSelectOrdinal = i18n._("{count, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}", {
      count: count
    });
    "#
);

to!(
    js_icu_diffrent_object_literal_syntax,
     r#"
         import { plural } from "@lingui/macro";

        const messagePlural = plural(count, {
           one: '# Book',
           "other": '# Books',
           few: ('# Books'),
        })
     "#,
    r#"
      import { i18n } from "@lingui/core";

      const messagePlural = i18n._("{count, plural, one {# Book} other {# Books} few {# Books}}", {
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

to!(
    js_icu_nested_in_t,
     r#"
        import { t, selectOrdinal } from '@lingui/macro'

        t`This is my ${selectOrdinal(count, {
          one: "st",
          two: "nd",
          other: "rd"
        })} cat`
     "#,
    r#"
      import { i18n } from "@lingui/core";

      i18n._("This is my {count, selectordinal, one {st} two {nd} other {rd}} cat", {
        count: count
      });
    "#
);

to!(
    js_icu_nested_in_choices,
     r#"
import { plural } from "@lingui/macro"
const message = plural(numBooks, {
   one: plural(numArticles, {
      one: `1 book and 1 article`,
      other: `1 book and ${numArticles} articles`,
   }),
   other: plural(numArticles, {
      one: `${numBooks} books and 1 article`,
      other: `${numBooks} books and ${numArticles} articles`,
   }),
})
     "#,
    r#"
import { i18n } from "@lingui/core"
const message = i18n._("{numBooks, plural, one {{numArticles, plural, one {1 book and 1 article} other {1 book and {numArticles} articles}}} other {{numArticles, plural, one {{numBooks} books and 1 article} other {{numBooks} books and {numArticles} articles}}}}", {
    numBooks: numBooks,
    numArticles: numArticles
})
    "#
);

to!(
    js_icu_nesting_unsupported_call,
     r#"
import { plural } from "@lingui/macro"
const message = plural(numBooks, {
   one: myFn(),
   other: 'Ola!'
})
     "#,
    r#"
import { i18n } from "@lingui/core"
const message = i18n._("{numBooks, plural, one {} other {Ola!}}", {
    numBooks: numBooks,
})
    "#
);