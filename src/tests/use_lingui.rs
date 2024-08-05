use crate::{to};

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
        const {  i18n: $__i18n, i18n } = $_useLingui();
        
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
    const { i18n: $__i18n } = $_useLingui();
    const a = $__i18n._({
        id: "xeiujy",
        message: "Text"
    });
}
    "#
);