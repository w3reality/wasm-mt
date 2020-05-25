use wasm_mt::prelude::*;
use wasm_mt::utils::console_ln;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        let pkg_js = "./pkg/executors.js";
        let mt = WasmMt::new(pkg_js).and_init().await.unwrap();

        let _ = run_executors(mt).await;
    });
}

async fn run_executors(mt: WasmMt) -> Result<(), JsValue> {
    // Prepare threads
    let mut v: Vec<wasm_mt::Thread> = vec![];
    for i in 0..4 {
        let th = mt.thread().and_init().await?;
        th.set_id(&i.to_string());
        v.push(th);
    }

    console_ln!("🔥 serial executor:");
    for th in &v {
        console_ln!("starting a thread");
        let ans = exec!(th, move || Ok(JsValue::from(42))).await?;
        console_ln!("ans: {:?}", ans);
    }

    console_ln!("🔥 parallel executor:");
    for th in v {
        spawn_local(async move {
            console_ln!("starting a thread");
            let ans = exec!(th, move || Ok(JsValue::from(42))).await.unwrap();
            console_ln!("ans: {:?}", ans);
        });
    }

    Ok(())
}
