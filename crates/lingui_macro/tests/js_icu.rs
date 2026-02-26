#[macro_use]
mod common;

to!(
    js_icu_macro,
    r#"
    import { plural, select, selectOrdinal } from "@lingui/core/macro";
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
     "#
);

to!(
    js_icu_diffrent_object_literal_syntax,
    r#"
        import { plural } from "@lingui/core/macro";

        const messagePlural = plural(count, {
           one: '# Book',
           "other": '# Books',
           few: ('# Books'),
        })
     "#
);

to!(
    js_choices_may_contain_expressions,
    r#"
import { plural, select, selectOrdinal } from "@lingui/core/macro";
const messagePlural = plural(count, {
   one: foo.bar,
   other: variable
})
const messageSelect = select(gender, {
   male: 'he',
   female: variable,
   third: fn(),
   other: foo.bar
})
     "#
);

to!(
    js_should_not_touch_non_lungui_fns,
    r#"
    import { plural } from "@lingui/core/macro";
    const messagePlural = customName(count, {
       one: '# Book',
       other: '# Books'
    })
     "#
);

to!(
    js_plural_with_placeholders,
    r#"
       import { plural } from "@lingui/core/macro";

       const message = plural(count, {
           one: `${name} has # friend`,
           other: `${name} has # friends`
        })
     "#
);

to!(
    js_dedup_values_in_icu,
    r#"
       import { plural } from "@lingui/core/macro";

       const message = plural(count, {
           one: `${name} has ${count} friend`,
           other: `${name} has {count} friends`
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
     "#
);

to!(
    js_icu_nested_in_choices,
    r#"
import { plural } from "@lingui/core/macro"
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
     "#
);

to!(
    js_plural_with_offset_and_exact_matches,
    r#"
        import { plural } from '@lingui/macro'
        plural(users.length, {
          offset: 1,
          0: "No books",
          1: "1 book",
          other: "\# books"
        });
     "#
);

to!(
    js_should_not_treat_offset_in_select,
    r#"
        import { select } from '@lingui/macro'
        select(value, {
          offset: "..",
          any: "..",
          other: "..",
        });
     "#
);
