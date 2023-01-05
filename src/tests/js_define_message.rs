use crate::{to};

to!(
    should_transform_define_message,
     r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          comment: "Description",
          message: "Message"
        })
     "#,
    r#"
        const message = {
          comment: "Description",
          id: "Message"
        }
    "#
);

to!(
    should_preserve_custom_id,
     r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          comment: "Description",
          id: "custom.id",
          message: "Message",
        })
     "#,
    r#"
        const message = {
          comment: "Description",
          id: "custom.id",
          message: "Message"
        }
    "#
);

to!(
    should_expand_values,
     r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          message: `Hello ${name}`
        })
     "#,
    r#"
        const message = {
          id: "Hello {name}",
          values: {
            name: name,
          }
        }
    "#
);

to!(
    should_expand_macros,
     r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          comment: "Description",
          message: plural(count, { one: "book", other: "books" })
        })
     "#,
    r#"
        const message = {
          comment: "Description",
          id: "{count, plural, one {book} other {books}}",
          values: {
            count: count
          }
        }
    "#
);

// to!(
// ,
//      r#"
//
//      "#,
//     r#"
//     "#
// );
