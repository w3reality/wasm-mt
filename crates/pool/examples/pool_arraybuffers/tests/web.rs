extern crate pool_arraybuffers;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn app_pool_arraybuffers() {
    pool_arraybuffers::run(true).await.unwrap();
}
