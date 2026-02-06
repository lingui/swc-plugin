import { useLingui as $_useLingui } from "@lingui/react";
import { useLingui } from '@lingui/react';
function MyComponent() {
    const { _ } = useLingui();
    console.log(_);
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._(/*i18n*/ {
        __lingui__: true,
        id: "xeiujy",
        message: "Text"
    });
}
