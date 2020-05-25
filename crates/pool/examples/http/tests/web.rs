extern crate http;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_mt_pool_test::create_pool;

#[wasm_bindgen_test]
async fn app_http() {
    let _ = http::run(create_pool(2).await).await;
}
