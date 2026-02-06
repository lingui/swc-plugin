import { useLingui as $_useLingui } from "@lingui/react";
function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const a = $__i18n._(/*i18n*/ {
        __lingui__: true,
        id: "hJRCh6",
        message: "Text {0, plural, offset:1 =0 {No books} =1 {1 book} other {# books}}",
        values: {
            0: users.length
        }
    });
}
