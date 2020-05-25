#![feature(async_closure)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, Clamped};
use wasm_bindgen_futures::spawn_local;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use wasm_mt::prelude::*;
use wasm_mt::utils::{console_ln, Counter};
use std::rc::Rc;

mod julia_set;

#[wasm_bindgen]
pub fn app() {
    spawn_local(async move {
        let mt = WasmMt::new("./pkg/parallel.js").and_init().await.unwrap();
        let _ = run(mt).await;
    });
}

fn get_canvas_context(id: &str) -> CanvasRenderingContext2d {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(id).unwrap();
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    ctx
}

fn compute_image(
    width: u32,
    height: u32,
    use_arraybuffer: bool,
) -> Result<JsValue, JsValue> {
    julia_set::compute(
        width, height,
        0.00375, // scale adjusted for 800x800
        -0.15, 0.65, // C
        use_arraybuffer)
}

fn draw_image(
    ctx: &CanvasRenderingContext2d,
    data: &JsValue,
    width: u32,
    height: u32,
    use_arraybuffer: bool,
) {
    if use_arraybuffer {
        let ab = data.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
        let mut vec = js_sys::Uint8Array::new(ab).to_vec();
        let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut vec), width, height)
            .unwrap()
            .into();
        ctx.put_image_data(&data, 0.0, 0.0).unwrap();
    } else {
        ctx.put_image_data(
            data.dyn_ref::<ImageData>().unwrap(),
            0.0, 0.0).unwrap();
    };
}

async fn run_task(th: &wasm_mt::Thread) {
    let width: u32 = 800;
    let height: u32 = 800;

    let th_id = th.get_id().unwrap();
    console_ln!("th_{}: starting", th_id);

    let use_arraybuffer = true;

    let data = if use_arraybuffer {
        // `ArrayBuffer` workaround
        exec!(th, move || compute_image(width, height, use_arraybuffer))
            .await.unwrap()

        // TODO Support 'transfer' functionality in `wasm_mt`. (That's not the bottle
        // of this example app though.)
    } else {
        // FIXME !!!!
        //
        exec!(th, move || compute_image(width, height, use_arraybuffer))
            .await.unwrap()
        //
        // On Chrome/Opera, `debug_ln!()` shows
        //   on_message(): msg: JsValue(null); oops, `.await` will hang!!
        // On the contrary, an `ImageData` created via JavaScript below works though.
        // It seems? there's something odd going on inside
        //   `web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(...`
        // TODO check.
        //
        // exec_js!(th, "
        //     // https://developer.mozilla.org/en-US/docs/Web/API/ImageData/ImageData
        //     const arr = new Uint8ClampedArray(4 * 800 * 800);
        //     for (let i = 0; i < arr.length; i += 4) {
        //         arr[i + 0] = 0;    // R value
        //         arr[i + 1] = 190;  // G value
        //         arr[i + 2] = 0;    // B value
        //         arr[i + 3] = 255;  // A value
        //     }
        //     let imageData = new ImageData(arr, 800);
        //     return imageData;
        // ").await.unwrap()
    };
    // console_ln!("data: {:?}", data);

    console_ln!("th_{}: done", th_id);

    let ctx = get_canvas_context("drawing");
    draw_image(&ctx, &data, width, height, use_arraybuffer);
}

// main thread
pub async fn run(mt: WasmMt) -> Result<(), JsValue> {
    // Instead of putting
    //   <canvas id="drawing" width="800" height="800"></canvas>
    // in index.html, dynamically appending a new canvas for
    // `wasm_bindgen_test` in tests/web.rs.
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    document.body().unwrap().append_child(&canvas)?;
    canvas.set_width(800);
    canvas.set_height(800);
    canvas.set_id("drawing");

    let num = 4;

    // Prepare threads

    let mut v: Vec<wasm_mt::Thread> = vec![];
    for i in 0..num {
        let th = mt.thread().and_init().await?;
        th.set_id(&i.to_string());
        v.push(th);
    }

    // Serial executor

    let perf = web_sys::window().unwrap().performance().unwrap();
    let time_start = perf.now();
    for i in 0..num {
        run_task(&v[i]).await;
    }
    console_ln!("serial executor: {} tasks in {:.2}ms", num, perf.now() - time_start);

    // Parallel executor

    let time_start = perf.now();
    let count = Rc::new(Counter::new());
    for th in v {
        let count = count.clone();
        spawn_local(async move {
            run_task(&th).await;

            if count.inc() == num {
                let perf = web_sys::window().unwrap().performance().unwrap();
                console_ln!("parallel executor {} tasks in {:.2}ms", num, perf.now() - time_start);
            }
        });
    }
    //====
    // v.into_iter().for_each(|th| spawn_local(async move {
    //     run_task(&th).await;
    // }));

    Ok(())
}
