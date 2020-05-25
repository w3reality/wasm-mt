#![feature(async_closure)]

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_bindgen::prelude::*;
use wasm_mt::{utils, utils::console_ln};
use wasm_mt_pool::prelude::*;
use wasm_mt_pool_test::create_pool;

async fn sleep(ms: u32) {
    console_ln!("sleeping for {}ms...", ms);
    utils::sleep(ms).await;
    console_ln!("JUST WOKE UP after {}ms!!", ms)
}

type ResultJJ = Result<JsValue, JsValue>;

#[wasm_bindgen_test]
async fn basics() {
    let pool = create_pool(2).await;

    for _ in 0..2 { // parallel
        pool.exec(FnOnce!(move || Ok(JsValue::from(42))));
        pool_exec!(pool, move || Ok(JsValue::from(42)));
    }
    for _ in 0..2 { // parallel
        pool.exec_async(FnOnce!(async move || Ok(JsValue::from(42))));
        pool_exec!(pool, async move || Ok(JsValue::from(42)));
    }

    let cb = move |result: ResultJJ| {
        // console_ln!("callback: result: {:?}", result);
        assert_eq!(result.unwrap(), JsValue::from(42));
    };
    for _ in 0..2 { // parallel
        pool.exec_with_cb(FnOnce!(move || Ok(JsValue::from(42))), cb);
        pool_exec!(pool, move || Ok(JsValue::from(42)), cb);
    }
    for _ in 0..2 { // parallel
        pool.exec_async_with_cb(FnOnce!(async move || Ok(JsValue::from(42))), cb);
        pool_exec!(pool, async move || Ok(JsValue::from(42)), cb);
    }

    sleep(500).await;
    assert_eq!(pool.count_pending_jobs(), 0);
}

#[wasm_bindgen_test]
async fn basics_js() {
    let pool = create_pool(2).await;
    let js = "const add = (x, y) => x + y; return add(1, 2);";
    let js_async = "const addAsync = (x, y) => new Promise(res => setTimeout(() => res(x + y), 10)); return await addAsync(1, 2);";

    for _ in 0..2 { // parallel
        pool.exec_js(js);
        pool_exec_js!(pool, js);
    }
    for _ in 0..2 { // parallel
        pool.exec_js_async(js_async);
        pool_exec_js_async!(pool, js_async);
    }

    let cb = move |result: ResultJJ| {
        // console_ln!("callback: result: {:?}", result);
        assert_eq!(result.unwrap(), JsValue::from(3));
    };
    for _ in 0..2 { // parallel
        pool.exec_js_with_cb(js, cb);
        pool_exec_js!(pool, js, cb);
    }
    for _ in 0..2 { // parallel
        pool.exec_js_async_with_cb(js_async, cb);
        pool_exec_js_async!(pool, js_async, cb);
    }

    sleep(500).await;
    assert_eq!(pool.count_pending_jobs(), 0);
}

#[wasm_bindgen_test]
async fn pool() {
    {
        let pool = create_pool(2).await;

        for _ in 0..4 { // parallel macro calls
            pool_exec!(pool, move || Ok(JsValue::from("m")));
        }
        for _ in 0..4 { // parallel bare calls
            pool.exec(FnOnce!(move || Ok(JsValue::from("b"))));
        }

        sleep(500).await;
        assert_eq!(pool.count_pending_jobs(), 0);

        console_ln!("`pool` is being dropped!!");
    }
}
