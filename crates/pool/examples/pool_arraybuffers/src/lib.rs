#![feature(async_closure)]

use wasm_mt_pool::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, sleep};
use js_sys::ArrayBuffer;

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        let ab_js = fetch_as_arraybuffer("./pkg/pool_arraybuffers.js").await.unwrap();
        let ab_wasm = fetch_as_arraybuffer("./pkg/pool_arraybuffers_bg.wasm").await.unwrap();
        run(ab_js, ab_wasm).await.unwrap();
    });
}

pub async fn run(ab_js: ArrayBuffer, ab_wasm: ArrayBuffer) -> Result<(), JsValue> {
    let size = 2;
    let pool = ThreadPool::new_with_arraybuffers(size, ab_js, ab_wasm)
        .and_init().await?;
    console_ln!("pool with {} threads is ready now!", size);

    let num = 4;
    console_ln!("{} closures:", num);
    for idx in 0..num {
        pool_exec!(pool, move || {
            console_ln!("idx: {}", idx); // not necessarily ordered
            Ok(JsValue::NULL)
        });
    }

    sleep(2_000).await; // Do sleep long enough to ensure all jobs are completed.
    assert_eq!(pool.count_pending_jobs(), 0);

    console_ln!("pool is getting dropped.");

    Ok(())
}
