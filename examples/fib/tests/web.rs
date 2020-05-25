extern crate fib;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_mt_test::{create_ab_init, get_pkg_js_uri};

#[wasm_bindgen_test]
async fn app_fib() {
    let pkg_js_uri = get_pkg_js_uri();
    let ab = create_ab_init(&pkg_js_uri).await.unwrap();
    let _ = fib::run(&pkg_js_uri, Some(ab)).await;
}
