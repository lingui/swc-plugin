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
