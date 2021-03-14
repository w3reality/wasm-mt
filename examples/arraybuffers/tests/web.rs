extern crate arraybuffers;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use wasm_mt_test::{get_pkg_js_uri, pkg_js_no_modules_from};
use wasm_mt::utils::{ab_from_text, fetch_as_arraybuffer, fetch_as_text};

#[wasm_bindgen_test]
async fn app_arraybuffers() {
    let pkg_js_uri = get_pkg_js_uri(); // e.g. http://127.0.0.1:8000/wasm-bindgen-test
    let pkg_wasm_uri = format!("{}_bg.wasm", pkg_js_uri);

    let ab_js = ab_from_text(
        &pkg_js_no_modules_from(
            &fetch_as_text(&pkg_js_uri).await.unwrap()));
    let ab_wasm = fetch_as_arraybuffer(&pkg_wasm_uri).await.unwrap();

    arraybuffers::run(ab_js, ab_wasm).await.unwrap();
}
