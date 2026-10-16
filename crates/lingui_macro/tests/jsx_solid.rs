use lingui_macro::{JsxRuntime, LinguiOptions};

#[macro_use]
mod common;

fn solid() -> LinguiOptions {
    LinguiOptions {
        jsx_runtime: JsxRuntime::Solid,
        ..Default::default()
    }
}

to!(
    solid_basic_component_with_attrs,
    solid(),
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello <a href="/docs">docs</a>.</Trans>;
    "#
);

to!(
    solid_self_closing_element,
    solid(),
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Hello<br />world</Trans>;
    "#
);

to!(
    solid_nested_elements_and_values,
    solid(),
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  <p>Hello <a href="/u"><em>{name}</em></a></p>
</Trans>;
    "#
);

to!(
    solid_member_expression_tag,
    solid(),
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Click <Foo.Bar kind="x">here</Foo.Bar></Trans>;
    "#
);

to!(
    solid_existing_spread_attrs,
    solid(),
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>Go <a {...rest} href="/x">there</a></Trans>;
    "#
);

to!(
    solid_named_placeholder_reused,
    LinguiOptions {
        jsx_runtime: JsxRuntime::Solid,
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans>
  <a _t="link" href="/a">first</a> and <a _t="link" href="/a">second</a>
</Trans>;
    "#
);

to!(
    solid_plural_inside_trans,
    solid(),
    r##"
import { Trans, Plural } from "@lingui/react/macro";
<Trans>
  Hello <b>world</b>, <Plural value={count} one="# item" other="# items" />
</Trans>;
    "##
);

to!(
    solid_element_inside_icu_case,
    solid(),
    r#"
import { Trans, Plural } from "@lingui/react/macro";
<Trans>
  <Plural value={count} one={<a href="/one">one</a>} other="many" />
</Trans>;
    "#
);

to!(
    solid_does_not_affect_t_and_use_lingui,
    solid(),
    r#"
import { t } from "@lingui/core/macro";
import { useLingui } from "@lingui/react/macro";
const a = t`Hello ${name}`;
function C() {
  const { t } = useLingui();
  return t`Hi ${name}`;
}
    "#
);

to!(
    solid_props_name_collision,
    solid(),
    r#"
import { Trans } from "@lingui/react/macro";
function C(_props) {
  return <Trans>Hello <a href={_props.href}>docs</a>.</Trans>;
}
    "#
);

to_panic!(
    solid_distinct_elements_same_placeholder,
    LinguiOptions {
        jsx_runtime: JsxRuntime::Solid,
        jsx_placeholder_attribute: Some("_t".into()),
        ..Default::default()
    },
    r#"
import { Trans } from "@lingui/react/macro";
<Trans><a _t="link" href="/a">one</a> and <a _t="link" href="/b">two</a></Trans>;
    "#
);
