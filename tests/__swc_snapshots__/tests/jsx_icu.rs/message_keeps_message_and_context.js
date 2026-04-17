import { Trans as Trans_ } from "@lingui/react";
<Trans_ {.../*i18n*/ {
    id: "custom.id",
    values: {
        count: count
    },
    components: {
        0: <a href="/more"/>
    },
    message: "{count, plural, offset:1 =0 {Zero items} other {<0>A lot of them</0>}}",
    context: "My Context"
}} render="render" i18n="i18n"/>;
