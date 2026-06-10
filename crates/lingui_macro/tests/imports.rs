#[macro_use]
mod common;

to!(
    should_add_not_clashing_imports,
    r#"
       import { t } from "@lingui/core/macro";
       import { Plural } from "@lingui/react/macro";
       import { i18n } from "@lingui/core";
       import { Trans } from "@lingui/react";

       t`Test`;
       <Plural value={value} one="..." other="..."/>;
       <Trans>Untouched</Trans>
     "#
);

to!(
    jsx_should_process_only_elements_imported_from_macro,
    r#"
      import { Plural } from "@lingui/react/macro";
      import { Select } from "./my-select-cmp";

      ;<Plural
       value={count}
       one="Message"
       other="Messages"
      />

      ;<Select prop="propValue">Should be untouched</Select>
     "#
);

to!(
    jsx_should_process_only_elements_imported_from_macro2,
    r#"
      import { Trans } from "@lingui/react";
      import { Plural } from "@lingui/react/macro";

      ;<Plural
       value={count}
       one="Message"
       other="Messages"
      />

      ;<Trans>Should be untouched</Trans>
     "#
);

to!(
    js_should_process_only_elements_imported_from_macro,
    r#"
      import { plural } from "@lingui/core/macro";
      import { t } from "./custom-t";

       t`Don't touch me!`
       plural(value, {one: "...", other: "..."})
     "#
);

to!(
    js_should_process_only_elements_imported_from_macro2,
    r#"
      import { t } from "@lingui/core/macro";
      import { plural } from "./custom-plural";

       t`Hello World!`;
       plural(value, {one: "...", other: "..."});
     "#
);

to!(
    js_should_support_renamed_imports,
    r#"
      import { t as i18nT, plural as i18nPlural } from "@lingui/core/macro";

       i18nT`Hello World!`;
       i18nPlural(value, {one: "...", other: "..."});
     "#
);

to!(
    jsx_should_support_renamed_imports,
    r#"
      import { Trans as I18nTrans, Plural as I18nPlural } from "@lingui/react/macro";

      ;<I18nPlural
       value={count}
       one="Message"
       other="Messages"
      />

      ;<I18nTrans>Hello!</I18nTrans>
     "#
);

to!(
    should_add_imports_after_directive_prologues,
    r#"
     "use client";
      import { t } from "@lingui/core/macro"
      import foo from "bar"
      t`Text`
     "#
);

to!(
    should_transform_custom_core_macro_package,
    lingui_macro::LinguiOptions {
        macro_packages: lingui_macro::MacroPackagesConfig {
            core: vec!["@acme/core/macro".into()],
            jsx: vec!["@lingui/macro".into(), "@lingui/react/macro".into()],
        },
        ..Default::default()
    },
    r#"
    import { t } from "@acme/core/macro";

    t`Hello World!`;
  "#
);

to!(
    should_transform_custom_jsx_macro_package,
    lingui_macro::LinguiOptions {
        macro_packages: lingui_macro::MacroPackagesConfig {
            core: vec!["@lingui/macro".into(), "@lingui/core/macro".into()],
            jsx: vec!["@acme/react/macro".into()],
        },
        ..Default::default()
    },
    r#"
    import { Trans } from "@acme/react/macro";

    ;<Trans>Hello!</Trans>
  "#
);

to!(
    should_transform_messages_after_ts_module_declarations,
    r#"
import { t } from "@lingui/core/macro";
import { Trans } from "@lingui/react/macro";

declare module "x" {
  interface I {
    a: string;
  }
}

const afterModule = <Trans>after module</Trans>;

declare global {
  interface Window {
    b: string;
  }
}

const afterGlobal = t`after global`;

declare namespace Foo {
  interface I {
    c: string;
  }
}

const afterDeclareNamespace = <Trans>after declare namespace</Trans>;

namespace Bar {
  export interface I {
    d: string;
  }
}

const afterNamespace = t`after namespace`;
     "#
);
