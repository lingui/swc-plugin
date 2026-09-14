use lingui_macro::{DescriptorFields, LinguiOptions};

#[macro_use]
mod common;

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
    js_t_with_top_of_file_directive_before_macro_import,
    r#"
        // lingui-set context="test"
        import { t } from '@lingui/core/macro';
        const msg = t`Success`
    "#
);

to!(
    js_t_with_directive_before_removed_macro_import_after_regular_import,
    r#"
        import foo from 'bar';
        /* lingui-set context="test" */
        import { t } from '@lingui/core/macro';
        const msg = t`Success`
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

to!(
    js_plural_with_directive_context,
    r##"
      import { plural } from "@lingui/core/macro";
      /* lingui-set context="my context" */
      const msg = plural(count, { one: "# book", other: "# books" })
   "##
);

to!(
    js_select_with_directive_context,
    r#"
      import { select } from "@lingui/core/macro";
      /* lingui-set context="my context" */
      const msg = select(gender, { male: "he", female: "she", other: "they" })
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

to!(
    jsx_trans_with_unclosed_quotes,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
      // lingui-set idPrefix="root"
      import type { MessageDescriptor } from '@lingui/core'
      import { msg, t } from '@lingui/core/macro'
      import { Trans } from '@lingui/react/macro'

      const X = () => <p>'</p>
      const Y = () => <p>`</p>

      // lingui-set idPrefix="different"
      const different = {
        a: msg({ id: '.a', message: `different a` }),
        b: msg({ id: '.b', message: `different b` }),
      } as const satisfies Record<string, MessageDescriptor>
    "#
);

to!(
    jsx_trans_with_directive_context,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set context="my context" */
       const el = <Trans>Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_directive_comment,
    r#"
       import { Trans } from "@lingui/react/macro";
       // lingui-set comment="translator note"
       const el = <Trans>Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_explicit_context_overrides_directive,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set context="directive ctx" */
       const el = <Trans context="explicit ctx">Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_directive_id_prefix_and_explicit_id,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set idPrefix="module." */
       const el = <Trans id="greeting">Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_directive_id_prefix_without_explicit_id,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set idPrefix="module." */
       const el = <Trans>Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_dynamic_id_and_no_id_prefix,
    r#"
       import { Trans } from "@lingui/react/macro";
       const dynId = "dynamic";
       const el = <Trans id={dynId}>Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_dynamic_id_and_id_prefix,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set idPrefix="module." */
       const el = <Trans id={dynId}>Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_id_prefix_leader,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set idPrefix="checkout" */
       const el1 = <Trans id=".usesPrefix">Hello</Trans>;
       const el2 = <Trans id=".usesPrefix.with.subpath">Hello</Trans>;
       const el3 = <Trans id="unprefixed.key">Hello</Trans>;
       const el4 = <Trans id="unprefixed">Hello</Trans>;
       const el5 = <Trans id="test">Hello</Trans>;
     "#
);

to!(
    jsx_trans_with_matching_id_prefix_leader,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
         import { Trans } from "@lingui/react/macro";
         /* lingui-set idPrefix="checkout" */
         const el = <Trans id=".greeting">Hello</Trans>;
       "#
);

to!(
    jsx_trans_with_non_matching_id_prefix_leader,
    LinguiOptions {
        id_prefix_leader: Some(".".into()),
        ..Default::default()
    },
    r#"
         import { Trans } from "@lingui/react/macro";
         /* lingui-set idPrefix="checkout." */
         const el = <Trans id="greeting">Hello</Trans>;
       "#
);

to!(
    jsx_multiple_directives_switching_context_mid_file,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set context="header" */
       const h = <Trans>Title</Trans>;
       /* lingui-set context="footer" */
       const f = <Trans>Copyright</Trans>;
     "#
);

to!(
    jsx_plural_with_directive_context,
    r##"
       import { Plural } from "@lingui/react/macro";
       /* lingui-set context="my context" */
       const el = <Plural value={count} one="# book" other="# books" />;
     "##
);

to!(
    jsx_select_with_directive_context,
    r#"
       import { Select } from "@lingui/react/macro";
       /* lingui-set context="my context" */
       const el = <Select value={gender} male="he" female="she" other="they" />;
     "#
);

to!(
    use_lingui_t_with_directive_context,
    r#"
import { useLingui } from '@lingui/react/macro';

function App() {
  const { t } = useLingui();
  /* lingui-set context="my context" */
  return t`Hello`;
}
     "#
);

to!(
    use_lingui_t_with_directive_applied_per_reference,
    r#"
import { useLingui } from '@lingui/react/macro';

function App() {
  const { t } = useLingui();
  /* lingui-set context="first" */
  const msg1 = t`Hello`;
  /* lingui-set context="second" */
  const msg2 = t`World`;
  return msg1 + msg2;
}
     "#
);

// --- block-comment directives inside JSX (regressions vs. the regex scanner) ---

to!(
    jsx_trans_with_block_comment_reset_in_expression_container,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set context="header" */
       const a = <Trans>Title</Trans>;
       const el = (
         <div>
           {/* lingui-reset */}
           <Trans>Body</Trans>
         </div>
       );
     "#
);

to!(
    jsx_trans_with_block_comment_set_in_expression_container,
    r#"
       import { Trans } from "@lingui/react/macro";
       const el = (
         <div>
           {/* lingui-set context="section" */}
           <Trans>Title</Trans>
         </div>
       );
     "#
);

to!(
    jsx_block_comment_directive_with_trailing_code,
    r#"
       import { Trans } from "@lingui/react/macro";
       /* lingui-set context="ctx" */ const noop = 1;
       const el = <Trans>Hello</Trans>;
     "#
);

to!(
    jsdoc_block_comment_directive,
    r#"
        import { t } from '@lingui/core/macro';
        /** lingui-set context="jsdoc" */
        const msg = t`Hello`
    "#
);

// An invalid directive is reported as a diagnostic (exercises the error branch
// of the directive collector) rather than silently ignored.
to_panic!(
    invalid_directive_reports_diagnostic,
    Default::default(),
    r#"
        import { t } from '@lingui/core/macro';
        /* lingui-set context */
        const msg = t`Hello`
    "#
);
