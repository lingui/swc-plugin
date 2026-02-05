import { i18n as $_i18n } from "@lingui/core";
const messagePlural = $_i18n._({
    id: "l6reUi",
    message: "{count, plural, one {{0}} other {{variable}}}",
    values: {
        count: count,
        variable: variable,
        0: foo.bar
    }
});
const messageSelect = $_i18n._({
    id: "M4Fisk",
    message: "{gender, select, male {he} female {{variable}} third {{0}} other {{1}}}",
    values: {
        gender: gender,
        variable: variable,
        0: fn(),
        1: foo.bar
    }
});
