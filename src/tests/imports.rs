use crate::{to};

to!(
    should_not_add_extra_imports,
     r#"
       import { t } from "@lingui/macro";
       import { i18n } from "@lingui/core";
       import { Trans } from "@lingui/react";

       t`Test`;
       <Trans>Test</Trans>;
     "#,
    r#"
       import { i18n } from "@lingui/core";
       import { Trans } from "@lingui/react";

       i18n._("Test");
       <Trans id={"Test"}/>;
    "#
);