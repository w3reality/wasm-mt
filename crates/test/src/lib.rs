//! Utility for testing crates with [`wasm-mt`](https://crates.io/crates/wasm-mt).

#![feature(async_closure)]

use wasm_mt::{WasmMt, utils};
use wasm_bindgen::prelude::*;
use js_sys::{Array, ArrayBuffer, Object, Reflect};
use web_sys::TextEncoder;

// Per crates/web only; TODO generalize for crates/node
pub fn get_pkg_js_uri() -> String {
    let href = utils::run_js("return location.href;").unwrap().as_string().unwrap();
    format!("{}wasm-bindgen-test", href)
}

#[wasm_bindgen(module = "/pkg/babel-transform.js")]
extern "C" {
    fn transform(input: &str, config: &Object) -> Object;
}

fn babel_transform(input: &str) -> Option<String> {
    let config = Object::new();
    Reflect::set(config.as_ref(),
        &JsValue::from("presets"),
        &Array::of1(&JsValue::from("es2015"))).unwrap_throw();

    Reflect::get(&transform(input, &config), &JsValue::from("code"))
        .unwrap_throw()
        .as_string()
}

// Per crates/web only; TODO generalize for crates/node
pub async fn create_ab_init(pkg_js_uri: &str) -> Result<ArrayBuffer, JsValue> {
    // let output = babel_transform("let xx = () => {};");
    // console_ln!("output: {:?}", output);
    // assert!(output.unwrap().starts_with("\"use strict\""));

    let pkg_js = utils::fetch_as_text(pkg_js_uri).await?;
    // console_ln!("pkg_js: {}", &pkg_js);

    // Work around the `import { transform }` stuff that's breaking init_js use inside threads.
    // Just comment the offending line, e.g.: import { transform } from './snippets/wasm-mt-test-dad973d9634694d9/pkg/babel-transform.js';
    let pkg_js = pkg_js.replace("import { transform } from", "// ");

    // Work around `import.meta` that's breaking `babel_transform()` below
    let pkg_js = pkg_js.replace("import.meta.url", "''");
    // console_ln!("pkg_js.len(): {}", pkg_js.len());

    let pkg_js_es5 = babel_transform(&pkg_js).unwrap();
    // console_ln!("pkg_js_es5: {}", &pkg_js_es5);
    // console_ln!("pkg_js_es5.len(): {}", pkg_js_es5.len());

    let mut init_js = String::new();
    // init_js.push_str(&fix_nodejs); // TODO in case of tests/crates/node
    init_js.push_str("
        return () => {
            const exports = {};
    ");
    init_js.push_str(&pkg_js_es5);
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
