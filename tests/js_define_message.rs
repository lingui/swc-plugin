use lingui_macro_plugin::{DescriptorFields, LinguiOptions};

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
    id_only_should_keep_only_id,
    LinguiOptions {
        descriptor_fields: DescriptorFields::IdOnly,
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

to!(
    message_should_keep_message_and_context,
    LinguiOptions {
        descriptor_fields: DescriptorFields::Message,
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

to!(
    define_message_with_directive_context,
    r#"
        import { defineMessage } from '@lingui/core/macro';
        /* lingui-set context="my context" */
        const msg = defineMessage({ message: "Hello" })
    "#
);

to!(
    define_message_tagged_template_with_directive,
    r#"
        import { defineMessage } from '@lingui/core/macro';
        /* lingui-set context="my context" comment="note" */
        const msg = defineMessage`Hello`
    "#
);

to!(
    define_message_explicit_context_overrides_directive,
    r#"
        import { defineMessage } from '@lingui/core/macro';
        /* lingui-set context="directive ctx" comment="directive cmt" */
        const msg = defineMessage({ message: "Hello", context: "explicit ctx" })
    "#
);

to!(
    define_message_id_prefix_with_explicit_id,
    r#"
        import { defineMessage } from '@lingui/core/macro';
        /* lingui-set idPrefix="module." */
        const msg = defineMessage({ id: "greeting", message: "Hello" })
    "#
);

to!(
    define_message_id_prefix_leader_with_matching_explicit_id,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
        import { defineMessage } from '@lingui/core/macro';
        /* lingui-set idPrefix="module" */
        const msg = defineMessage({ id: ".greeting", message: "Hello" })
    "#
);

to!(
    define_message_id_prefix_leader_with_non_matching_explicit_id,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
        import { defineMessage } from '@lingui/core/macro';
        /* lingui-set idPrefix="module." */
        const msg = defineMessage({ id: "greeting", message: "Hello" })
    "#
);
