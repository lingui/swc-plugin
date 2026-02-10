import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._(/*i18n*/ {
        id: "xeiujy",
        message: "Text"
    });
    {
        // here is child scope with own "t" binding, shouldn't be processed
        const t = ()=>{};
        t`Text`;
    }
    {
        // here is child scope which should be processed, since 't' relates to outer scope
        $__i18n._(/*i18n*/ {
            id: "xeiujy",
            message: "Text"
        });
    }
}
