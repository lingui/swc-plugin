use lingui_macro_plugin::LinguiOptions;

#[macro_use]
mod common;

to!(
    should_transform_define_message,
    r#"
        import { defineMessage, msg } from '@lingui/macro';
        const message1 = defineMessage({
          comment: "Description",
          message: "Message"
        })
        const message2 = msg({
          comment: "Description",
          message: "Message"
        })
     "#
);

to!(
    define_message_should_support_template_literal,
    r#"
        import { defineMessage, msg } from '@lingui/macro';
        const message1 = defineMessage`Message`;
        const message2 = msg`Message`
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
     "#
);

to!(
    should_expand_values,
    r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          message: `Hello ${name}`
        })
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
     "#
);

to!(
    should_kept_only_essential_props,
    LinguiOptions {
        strip_non_essential_fields: true,
        ..Default::default()
    },
    r#"
        import { defineMessage } from '@lingui/macro'
        const message1 = defineMessage`Message`;
        const message2 = defineMessage({
            message: `Hello ${name}`,
            id: 'msgId',
            comment: 'description for translators',
            context: 'My Context',
        })
    "#
);
