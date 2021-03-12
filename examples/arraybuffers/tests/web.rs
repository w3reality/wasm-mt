extern crate arraybuffers;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn app_arraybuffers() {
    arraybuffers::run().await.unwrap();
}
