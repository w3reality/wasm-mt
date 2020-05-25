extern crate parallel;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_mt_test::{create_mt, get_pkg_js_uri};

#[wasm_bindgen_test]
async fn app_parallel() {
    let pkg_js_uri = get_pkg_js_uri();
    let mt = create_mt(&pkg_js_uri).await;
    let _ = parallel::run(mt).await;
}
