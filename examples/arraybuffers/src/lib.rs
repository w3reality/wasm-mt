#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, run_js};

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        run().await.unwrap();
    });
}

pub async fn run() -> Result<(), JsValue> {
    let mut href = run_js("return location.href;")?.as_string().unwrap();

    let pkg_uri = if href.contains("index.html") {
        href.replace("index.html", "pkg")
    } else {
        href.push_str("/pkg");
        href
    };
    console_ln!("pkg_uri: {}", pkg_uri);

    let mt = WasmMt::new_with_arraybuffers(
        fetch_as_arraybuffer(&format!("{}/arraybuffers.js", pkg_uri)).await?,
        fetch_as_arraybuffer(&format!("{}/arraybuffers_bg.wasm", pkg_uri)).await?);

    // Note: In this particular example, this `th` won't be runnable
    //   under the wasm-bindgen-test context since different JS bindings
    //   ('<pkg_uri>/arraybuffer.js' and '<uri>/wasm-bindgen-test') are being used.
    let th = mt.thread().and_init().await?;

    // Just test `Thread` object-wise integrity here.
    th.set_id("foo");
    let id = th.get_id().unwrap().to_string();
    assert_eq!(id, "foo");
    console_ln!("id: {}", id);

    Ok(())
}