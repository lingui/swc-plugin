import { Trans as Trans_ } from "@lingui/react";
// Without comment - expression gets index 0
<Trans_ {.../*i18n*/ {
    __lingui__: true,
    id: "HW7Brx",
    values: {
        0: getText()
    },
    components: {
        0: <Link/>
    },
    message: "Click here<0>{0}</0>"
}}/>;
// With comment before expression - expression should STILL get index 0
<Trans_ {.../*i18n*/ {
    __lingui__: true,
    id: "HW7Brx",
    values: {
        0: getText()
    },
    components: {
        0: <Link/>
    },
    message: "Click here<0>{0}</0>"
}}/>;
