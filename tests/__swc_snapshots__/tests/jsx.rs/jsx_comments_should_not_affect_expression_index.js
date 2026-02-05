import { Trans as Trans_ } from "@lingui/react";
// Without comment - expression gets index 0
<Trans_ message={"Click here<0>{0}</0>"} id={"HW7Brx"} values={{
    0: getText()
}} components={{
    0: <Link/>
}}/>;
// With comment before expression - expression should STILL get index 0
<Trans_ message={"Click here<0>{0}</0>"} id={"HW7Brx"} values={{
    0: getText()
}} components={{
    0: <Link/>
}}/>;
