#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, resolve_pkg_uri};

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        run(false).await.unwrap();
    });
}

pub async fn run(is_test: bool) -> Result<(), JsValue> {
    let pkg_uri = resolve_pkg_uri()?;
    console_ln!("pkg_uri: {}", pkg_uri);

    let mt = WasmMt::new_with_arraybuffers(
        fetch_as_arraybuffer(&format!("{}/arraybuffers.js", pkg_uri)).await?,
        fetch_as_arraybuffer(&format!("{}/arraybuffers_bg.wasm", pkg_uri)).await?);

    let th = mt.thread().and_init().await?;

    // Just test `Thread` object-wise integrity here.
    th.set_id("foo");
    let id = th.get_id().unwrap().to_string();
    console_ln!("id: {}", id);
    assert_eq!(id, "foo");

    if is_test {
        // Note: In this particular example, `th` won't be runnable
        //   under the 'wasm-bindgen-test' context since we are using
        //   different JS bindings: '<pkg_uri>/arraybuffer.js' in this fuction, and
        //   '<uri>/wasm-bindgen-test' from the test builder.
        console_ln!("`exec!()` check skipped accordingly.");
    } else {
        let ans = exec!(th, move || Ok(JsValue::from(42))).await;
        console_ln!("ans: {:?}", ans);
        assert_eq!(ans, Ok(JsValue::from(42)));
    }

    Ok(())
}