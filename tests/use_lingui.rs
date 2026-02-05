use common::to;

to!(
    js_use_lingui_hook,
    r#"
     import { useLingui } from "@lingui/react/macro";

     const bla1 = () => {
        console.log()
     }

      function bla() {
        const { t, i18n } = useLingui();
        t`Refresh inbox`;
      }
     "#
);

to!(
    support_renamed_destructuring,
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { t: _ } = useLingui();
  const a = _`Text`;
}
     "#
);

to!(
    should_process_macro_with_matching_name_in_correct_scopes,
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { t } = useLingui();
  const a = t`Text`;

  {
    // here is child scope with own "t" binding, shouldn't be processed
    const t = () => {};
    t`Text`;
  }
  {
    // here is child scope which should be processed, since 't' relates to outer scope
    t`Text`;
  }
}
     "#
);

to!(
    support_nested_macro,
    r#"
import { useLingui } from '@lingui/react/macro';
import { plural } from '@lingui/core/macro';

function MyComponent() {
  const { t } = useLingui();
  const a = t`Text ${plural(users.length, {
          offset: 1,
          0: "No books",
          1: "1 book",
          other: "\# books"
        })}`;
}
     "#
);

to!(
    support_nested_macro_when_in_arrow_function_issue_2095,
    r#"
import { plural } from '@lingui/core/macro'
import { useLingui } from '@lingui/react/macro'

const MyComponent = () => {
  const { t } = useLingui();
  const a = t`Text ${plural(users.length, {
          offset: 1,
          0: "No books",
          1: "1 book",
          other: "\# books"
        })}`;
}
     "#
);

to!(
    support_passing_t_variable_as_dependency,
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { t } = useLingui();
  const a = useMemo(() => t`Text`, [t]);
}
     "#
);

to!(
    work_when_t_is_not_used,
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { i18n } = useLingui();
  console.log(i18n);
}
     "#
);

to!(
    work_with_existing_use_lingui_statement,
    r#"
import { useLingui as useLinguiMacro } from '@lingui/react/macro';
import { useLingui } from '@lingui/react';

function MyComponent() {
  const { _ } = useLingui();

  console.log(_);
  const { t } = useLinguiMacro();
  const a = t`Text`;
}
     "#
);

to!(
    work_with_multiple_react_components,
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { t } = useLingui();
  const a = t`Text`;
}

function MyComponent2() {
  const { t } = useLingui();
  const b = t`Text`;
}
     "#
);

to!(
    work_with_components_defined_as_arrow_function,
    r#"
import { useLingui } from '@lingui/react/macro';

const MyComponent = () => {
  const { t } = useLingui();
  const a = t`Text`;
}
     "#
);
