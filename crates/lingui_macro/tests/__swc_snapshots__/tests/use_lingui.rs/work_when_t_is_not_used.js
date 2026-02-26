import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n, _: $__ } = $_useLingui();
    console.log(i18n);
}
