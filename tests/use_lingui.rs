use crate::to;

to!(
    js_use_lingui_hook,
    // input
    r#"
     import { useLingui } from "@lingui/react/macro";

     const bla1 = () => {
        console.log()
     }

      function bla() {
        const { t, i18n } = useLingui();
        t`Refresh inbox`;
      }
     "#,
    // output after transform
    r#"
     import { useLingui as $_useLingui } from "@lingui/react";

    const bla1 = ()=>{
        console.log();
    };

    function bla() {
        const { i18n: $__i18n, i18n, _: $__ } = $_useLingui();
        
         $__i18n._({
            id: "EsCV2T",
            message: "Refresh inbox"
        });
    }
    "#
);

to!(
    support_renamed_destructuring,
    // input
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { t: _ } = useLingui();
  const a = _`Text`;
}
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._({
        id: "xeiujy",
        message: "Text"
    });
}
    "#
);

to!(
    should_process_macro_with_matching_name_in_correct_scopes,
    // input
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
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._({
        id: "xeiujy",
        message: "Text"
    });
    {
        const t = ()=>{};
        t`Text`;
    }
    {
        $__i18n._({
            id: "xeiujy",
            message: "Text"
        });
    }
}
    "#
);

to!(
    support_nested_macro,
    // input
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
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._({
        id: "hJRCh6",
        message: "Text {0, plural, offset:1 =0 {No books} =1 {1 book} other {# books}}",
        values: {
            0: users.length
        }
    });
}
    "#
);

to!(
    support_nested_macro_when_in_arrow_function_issue_2095,
    // input
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
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
const MyComponent = () => {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._({
        id: "hJRCh6",
        message: "Text {0, plural, offset:1 =0 {No books} =1 {1 book} other {# books}}",
        values: {
            0: users.length
        }
    });
}
    "#
);

to!(
    support_passing_t_variable_as_dependency,
    // input
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { t } = useLingui();
  const a = useMemo(() => t`Text`, [t]);
}
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = useMemo(()=>$__i18n._({
            id: "xeiujy",
            message: "Text"
        }), [
        $__
    ]);
}
    "#
);

to!(
    work_when_t_is_not_used,
    // input
    r#"
import { useLingui } from '@lingui/react/macro';

function MyComponent() {
  const { i18n } = useLingui();
  console.log(i18n);
}
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n, _: $__ } = $_useLingui();
    console.log(i18n);
}
    "#
);

to!(
    work_with_existing_use_lingui_statement,
    // input
    r#"
import { useLingui as useLinguiMacro } from '@lingui/react/macro';
import { useLingui } from '@lingui/react';

function MyComponent() {
  const { _ } = useLingui();

  console.log(_);
  const { t } = useLinguiMacro();
  const a = t`Text`;
}
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
import { useLingui } from '@lingui/react';
function MyComponent() {
    const { _ } = useLingui();
    console.log(_);
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._({
        id: "xeiujy",
        message: "Text"
    });
}
    "#
);

to!(
    work_with_multiple_react_components,
    // input
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
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._({
        id: "xeiujy",
        message: "Text"
    });
}
function MyComponent2() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const b = $__i18n._({
        id: "xeiujy",
        message: "Text"
    });
}
    "#
);

to!(
    work_with_components_defined_as_arrow_function,
    // input
    r#"
import { useLingui } from '@lingui/react/macro';

const MyComponent = () => {
  const { t } = useLingui();
  const a = t`Text`;
}
     "#,
    // output after transform
    r#"
import { useLingui as $_useLingui } from "@lingui/react";
const MyComponent = ()=>{
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._({
        id: "xeiujy",
        message: "Text"
    });
};
    "#
);
