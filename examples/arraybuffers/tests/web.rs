extern crate arraybuffers;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_mt_test::get_arraybuffers;

#[wasm_bindgen_test]
async fn app_arraybuffers() {
    let (ab_js, ab_wasm) = get_arraybuffers().await.unwrap();
    arraybuffers::run(ab_js, ab_wasm).await.unwrap();
}
