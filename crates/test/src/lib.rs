//! Utility for testing crates with [`wasm-mt`](https://crates.io/crates/wasm-mt).

#![feature(async_closure)]

use wasm_mt::{WasmMt, utils};
// use wasm_mt::console_ln;
use wasm_bindgen::prelude::*;
use js_sys::ArrayBuffer;
use web_sys::TextEncoder;

mod transform;
use transform::swc_transform;

// Per crates/web only; TODO generalize for crates/node
pub fn get_pkg_js_uri() -> String {
    let href = utils::run_js("return location.href;").unwrap().as_string().unwrap();
    format!("{}wasm-bindgen-test", href)
}

// Per crates/web only; TODO generalize for crates/node
pub async fn create_ab_init(pkg_js_uri: &str) -> Result<ArrayBuffer, JsValue> {
    // let output = swc_transform("let yy = () => {}; export default yy;");
    // console_ln!("output: {:?}", output);
    // assert!(output.unwrap().starts_with("\"use strict\""));

    let pkg_js = utils::fetch_as_text(pkg_js_uri).await?;
    // console_ln!("pkg_js: {}", &pkg_js);

    // `import.meta` workaround
    let pkg_js = pkg_js.replace("import.meta.url", "''");
    // console_ln!("pkg_js.len(): {}", pkg_js.len());

    let pkg_js = swc_transform(&pkg_js).unwrap();
    // console_ln!("(transformed) pkg_js: {}", &pkg_js);
    // console_ln!("(transformed) pkg_js.len(): {}", pkg_js.len());

    let mut init_js = String::new();
    // init_js.push_str(&fix_nodejs); // TODO in case of tests/crates/node
    init_js.push_str("
        return () => {
            const exports = {};
    ");
    init_js.push_str(&pkg_js);
    init_js.push_str("
            return Object.assign(init, exports);
        };
    ");
    // console_ln!("init_js: {}", init_js);

    let ab_init = utils::u8arr_from_vec(
        &TextEncoder::new()?.encode_with_input(&init_js)).buffer();

    Ok(ab_init)
}

pub async fn create_mt(pkg_js_uri: &str) -> WasmMt {
    let mt = WasmMt::new(&pkg_js_uri).and_init().await.unwrap();
    let ab = create_ab_init(&pkg_js_uri).await.unwrap();
    mt.set_ab_init(ab);

    mt
}
