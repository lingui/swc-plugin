use crate::to;

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
