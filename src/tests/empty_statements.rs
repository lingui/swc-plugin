use crate::to;

// Test case 1: Empty statements should be removed from statement lists
to!(
    should_remove_empty_statements_from_blocks,
    r#"
import { t } from "@lingui/core/macro";

function test() {
    const msg = t`Hello`;
    ;
    ;
    const msg2 = t`World`;
    ;
}
    "#,
    r#"
import { i18n as $_i18n } from "@lingui/core";

function test() {
    const msg = $_i18n._({
        id: "uzTaYi",
        message: "Hello"
    });
    const msg2 = $_i18n._({
        id: "v+jdqd",
        message: "World"
    });
}
    "#
);

// Test case 2: Empty variable declarations should be removed
to!(
    should_remove_empty_var_declarations,
    r#"
import { useLingui } from "@lingui/react/macro";

function MyComponent() {
    const { t } = useLingui();

    // This creates an empty var declaration after transformation
    const bar = "test";

    const msg = t`Message`;
}
    "#,
    r#"
import { useLingui as $_useLingui } from "@lingui/react";

function MyComponent() {
    const { i18n: $__i18n, _: $__ } = $_useLingui();
    const bar = "test";
    const msg = $__i18n._({
        id: "xDAtGP",
        message: "Message"
    });
}
    "#
);

// Test case 3: Empty statements at module level should be removed
to!(
    should_remove_empty_statements_at_module_level,
    r#"
import { t } from "@lingui/core/macro";

;
;

const msg = t`Hello`;

;
;

export default msg;
    "#,
    r#"
import { i18n as $_i18n } from "@lingui/core";

const msg = $_i18n._({
    id: "uzTaYi",
    message: "Hello"
});

export default msg;
    "#
);

// Test case 4: Nested blocks with empty statements
to!(
    should_remove_empty_statements_in_nested_blocks,
    r#"
import { t } from "@lingui/core/macro";

function test() {
    if (true) {
        ;
        const msg = t`Hello`;
        ;
    }
    ;
}
    "#,
    r#"
import { i18n as $_i18n } from "@lingui/core";

function test() {
    if (true) {
        const msg = $_i18n._({
            id: "uzTaYi",
            message: "Hello"
        });
    }
}
    "#
);

// Test case 5: Arrow functions with empty statements
to!(
    should_remove_empty_statements_in_arrow_functions,
    r#"
import { t } from "@lingui/core/macro";

const fn = () => {
    ;
    const msg = t`Test`;
    ;
    return msg;
    ;
};
    "#,
    r#"
import { i18n as $_i18n } from "@lingui/core";

const fn = ()=>{
    const msg = $_i18n._({
        id: "NnH3pK",
        message: "Test"
    });
    return msg;
};
    "#
);

// Test case 6: Empty statements in loops
to!(
    should_remove_empty_statements_in_loops,
    r#"
import { t } from "@lingui/core/macro";

function test() {
    for (let i = 0; i < 10; i++) {
        ;
        const msg = t`Loop ${i}`;
        ;
    }
    ;
    while (true) {
        ;
        const msg2 = t`While`;
        ;
        break;
    }
}
    "#,
    r#"
import { i18n as $_i18n } from "@lingui/core";

function test() {
    for(let i = 0; i < 10; i++){
        const msg = $_i18n._({
            id: "c+yTqm",
            message: "Loop {i}",
            values: {
                i: i
            }
        });
    }
    while(true){
        const msg2 = $_i18n._({
            id: "FBBphD",
            message: "While"
        });
        break;
    }
}
    "#
);

// Test case 7: Empty statements in try-catch blocks
to!(
    should_remove_empty_statements_in_try_catch,
    r#"
import { t } from "@lingui/core/macro";

function test() {
    try {
        ;
        const msg = t`Try`;
        ;
    } catch (e) {
        ;
        const msg2 = t`Catch`;
        ;
    }
}
    "#,
    r#"
import { i18n as $_i18n } from "@lingui/core";

function test() {
    try {
        const msg = $_i18n._({
            id: "/7tku+",
            message: "Try"
        });
    } catch (e) {
        const msg2 = $_i18n._({
            id: "iKsG5F",
            message: "Catch"
        });
    }
}
    "#
);

// Test case 8: Empty statements in switch cases
to!(
    should_remove_empty_statements_in_switch,
    r#"
import { t } from "@lingui/core/macro";

function test(value) {
    switch (value) {
        case 1:
            ;
            const msg1 = t`One`;
            ;
            break;
        case 2:
            ;
            const msg2 = t`Two`;
            ;
            break;
    }
}
    "#,
    r#"
import { i18n as $_i18n } from "@lingui/core";

function test(value) {
    switch(value){
        case 1:
            const msg1 = $_i18n._({
                id: "/DFChQ",
                message: "One"
            });
            break;
        case 2:
            const msg2 = $_i18n._({
                id: "z9dG+S",
                message: "Two"
            });
            break;
    }
}
    "#
);

