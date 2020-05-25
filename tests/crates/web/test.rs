#![feature(async_closure)]

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_bindgen::prelude::*;
use wasm_mt::prelude::*;
use wasm_mt::{Thread, console_ln, utils};
use wasm_mt_test::{create_mt, get_pkg_js_uri};

async fn create_test_thread() -> Thread {
    let pkg_js_uri = get_pkg_js_uri();

    create_mt(&pkg_js_uri).await.thread().and_init().await.unwrap()
}

#[wasm_bindgen_test]
async fn basics() {
    let th = create_test_thread().await;
    let ok42 = Ok(JsValue::from(42));

    assert_eq!(exec!(th, move || Ok(JsValue::from(42))).await, ok42);
    assert_eq!(th.exec(FnOnce!(move || Ok(JsValue::from(42)))).await, ok42);

    assert_eq!(exec!(th, async move || Ok(JsValue::from(42))).await, ok42);
    assert_eq!(th.exec_async(FnOnce!(async move || Ok(JsValue::from(42)))).await, ok42);
}

#[wasm_bindgen_test]
async fn basics_js() {
    let ok3 = Ok(JsValue::from(3));

    assert_eq!(utils::run_js("return 3;"), ok3);
    assert_eq!(utils::run_js_async("return 3;").await, ok3);

    let th = create_test_thread().await;
    let js = "const add = (x, y) => x + y; return add(1, 2);";
    let js_async = "const addAsync = (x, y) => new Promise(res => setTimeout(() => res(x + y), 10)); return await addAsync(1, 2);";

    assert_eq!(th.exec_js(js).await, ok3);
    assert_eq!(exec_js!(th, js).await, ok3);

    assert_eq!(th.exec_js_async(js_async).await, ok3);
    assert_eq!(exec_js_async!(th, js_async).await, ok3);
}

#[wasm_bindgen_test]
async fn thread() {
    {
        let th = create_test_thread().await;

        // macro call
        let result = exec!(th, move || Ok(JsValue::from("m"))).await;
        assert_eq!(result, Ok(JsValue::from("m")));

        // bare call
        let result = th.exec(FnOnce!(move || Ok(JsValue::from("b")))).await;
        assert_eq!(result, Ok(JsValue::from("b")));

        // bare call with the spurious warnings eliminated
        let num = 42;
        let result = th.exec({ #[allow(warnings)] {
            FnOnce!(move || {
                Ok(JsValue::from(num))
            })
        }}).await;
        assert_eq!(result, Ok(JsValue::from(42)));

        console_ln!("`th` is being dropped!!");
    }
}
