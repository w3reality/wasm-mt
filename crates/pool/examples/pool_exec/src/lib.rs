#![feature(async_closure)]

use wasm_mt_pool::prelude::*;

use wasm_mt::utils::{console_ln, sleep};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        let size = 2;
        let pool = ThreadPool::new(size, "./pkg/pool_exec.js")
            .and_init().await.unwrap();

        console_ln!("pool with {} threads is ready now!", size);

        let _ = demo(&pool).await;
        let _ = demo_js(&pool).await;

        console_ln!("pool is getting dropped.");
    });
}

async fn demo(pool: &ThreadPool) -> Result<(), JsValue> {
    let num = 4;

    console_ln!("a) 💦 pool_exec! {} closures:", num);
    for _ in 0..num {
        pool_exec!(pool, move || {
            console_ln!("a) closure: done.");
            Ok(JsValue::NULL)
        });
    }

    console_ln!("b) 💦 pool_exec! {} async closures:", num);
    for _ in 0..num {
        pool_exec!(pool, async move || {
            sleep(1000).await;
            console_ln!("b) async closure: done.");
            Ok(JsValue::NULL)
        });
    }

    let cb = move |result| {
        console_ln!("callback: result: {:?}", result);
    };

    console_ln!("c) 💦 pool_exec! {} closures with callback:", num);
    for _ in 0..num {
        pool_exec!(pool, move || {
            console_ln!("c) closure: done.");
            Ok(JsValue::from("C"))
        }, cb);
    }

    console_ln!("d) 💦 pool_exec! {} async closures with callback:", num);
    for _ in 0..num {
        pool_exec!(pool, async move || {
            sleep(1000).await;
            console_ln!("d) async closure: done.");
            Ok(JsValue::from("D"))
        }, cb);
    }

    sleep(6_000).await; // Do sleep long enough to ensure all jobs are completed.
    assert_eq!(pool.count_pending_jobs(), 0);

    Ok(())
}

async fn demo_js(pool: &ThreadPool) -> Result<(), JsValue> {
    let num = 4;

    let js = "
        const add = (x, y) => x + y;
        return add(1, 2);
    ";
    let js_async = "
        const sub = (x, y) => new Promise(res => {
            setTimeout(() => res(x - y), 1000);
        });
        return await sub(1, 2);
    ";

    console_ln!("e) 💦 pool_exec_js!:");
    for _ in 0..num {
        pool_exec_js!(pool, js);
    }

    console_ln!("f) 💦 pool_exec_js_async!:");
    for _ in 0..num {
        pool_exec_js_async!(pool, js_async);
    }

    let cb = move |result| {
        console_ln!("callback: result: {:?}", result);
    };

    console_ln!("g) 💦 pool_exec_js! with callback:");
    for _ in 0..num {
        pool_exec_js!(pool, js, cb);
    }

    console_ln!("h) 💦 pool_exec_js_async! with callback:");
    for _ in 0..num {
        pool_exec_js_async!(pool, js_async, cb);
    }

    sleep(6_000).await;
    assert_eq!(pool.count_pending_jobs(), 0);

    Ok(())
}
