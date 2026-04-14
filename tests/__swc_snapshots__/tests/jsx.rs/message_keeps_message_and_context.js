import { Trans as Trans_ } from "@lingui/react";
<Trans_ {.../*i18n*/ {
    id: "msg.hello",
    values: {
        name: name
    },
    components: {
        0: <strong/>
    },
    message: "Hello <0>{name}</0>",
    context: "My Context"
}} render="render" i18n="i18n"/>;
