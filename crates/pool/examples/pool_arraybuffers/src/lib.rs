#![feature(async_closure)]

use wasm_mt_pool::prelude::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_mt::utils::{console_ln, fetch_as_arraybuffer, resolve_pkg_uri, sleep};

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        run(false).await.unwrap();
    });
}

pub async fn run(is_test: bool) -> Result<(), JsValue> {
    let pkg_uri = resolve_pkg_uri()?;
    console_ln!("pkg_uri: {}", pkg_uri);

    let size = 2;
    let pool = ThreadPool::new_with_arraybuffers(size,
        fetch_as_arraybuffer(&format!("{}/pool_arraybuffers.js", pkg_uri)).await?,
        fetch_as_arraybuffer(&format!("{}/pool_arraybuffers_bg.wasm", pkg_uri)).await?)
        .and_init().await?;


    // let pool = ThreadPool::new(size, "./pkg/pool_arraybuffers.js")
    //     .and_init().await?;
    console_ln!("pool with {} threads is ready now!", size);


    if is_test {
        console_ln!("`poo_exec!()` check skipped accordingly.");
    } else {
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
    }

    console_ln!("pool is getting dropped.");

    Ok(())
}
