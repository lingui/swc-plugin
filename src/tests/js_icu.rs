use crate::to;

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
    const messagePlural = i18n._({
        id: "V/M0Vc",
        message: "{count, plural, one {# Book} other {# Books}}",
        values: {
            count: count
        }
    });
    const messageSelect = i18n._({
        id: "VRptzI",
        message: "{gender, select, male {he} female {she} other {they}}",
        values: {
            gender: gender
        }
    });
    const messageSelectOrdinal = i18n._({
        id: "Q9Q8Bj",
        message: "{count, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}",
        values: {
            count: count
        }
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

      const messagePlural = i18n._({
          id: "2y/Fr5",
          message: "{count, plural, one {# Book} other {# Books} few {# Books}}",
          values: {
              count: count
          }
      });
    "#
);

to!(
  js_choices_may_contain_expressions,
  r#"
import { plural, select, selectOrdinal } from "@lingui/macro";
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
     "#,
  r#"
import { i18n } from "@lingui/core";
const messagePlural = i18n._({
    id: "l6reUi",
    message: "{count, plural, one {{0}} other {{variable}}}",
    values: {
        count: count,
        variable: variable,
        0: foo.bar
    }
});
const messageSelect = i18n._({
    id: "M4Fisk",
    message: "{gender, select, male {he} female {{variable}} third {{0}} other {{1}}}",
    values: {
        gender: gender,
        variable: variable,
        0: fn(),
        1: foo.bar
    }
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
    const message = i18n._({
        id: "CvuUwE",
        message: "{count, plural, one {{name} has # friend} other {{name} has # friends}}",
        values: {
            count: count,
            name: name
        }
    });
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

    const message = i18n._({
        id: "tK7kAV",
        message: "{count, plural, one {{name} has {count} friend} other {{name} has {count} friends}}",
        values: {
            count: count,
            name: name
        }
    });
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

      i18n._({
          id: "LF3Ndn",
          message: "This is my {count, selectordinal, one {st} two {nd} other {rd}} cat",
          values: {
              count: count
          }
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
const message = i18n._({
    id: "AA3wsz",
    message: "{numBooks, plural, one {{numArticles, plural, one {1 book and 1 article} other {1 book and {numArticles} articles}}} other {{numArticles, plural, one {{numBooks} books and 1 article} other {{numBooks} books and {numArticles} articles}}}}",
    values: {
        numBooks: numBooks,
        numArticles: numArticles
    }
});
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
     "#,
  r#"
      import { i18n } from "@lingui/core";
      i18n._({
          id: "CF5t+7",
          message: "{0, plural, offset:1 =0 {No books} =1 {1 book} other {# books}}",
          values: {
              0: users.length
          }
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
     "#,
  r#"
      import { i18n } from "@lingui/core";
      i18n._({
          id: "QHtFym",
          message: "{value, select, offset {..} any {..} other {..}}",
          values: {
              value: value
          }
      });
    "#
);
