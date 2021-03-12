use crate::debug_ln;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{WorkerGlobalScope, MessageEvent};
use super::atw::{ThreadWorker as AtwThreadWorker, atw_decode_req_msg};
use super::decode_task_msg;
use super::job;

#[allow(dead_code)]
#[wasm_bindgen]
pub fn wmt_bootstrap(wgs: WorkerGlobalScope, req_id: &str) -> _Worker {
    let worker = _Worker::new(wgs);
    worker.atw_thw.send_response(req_id, &JsValue::from("bootstrap COMPLETE"), None);

    worker
}

#[wasm_bindgen]
pub struct _Worker {
    atw_thw: Rc<AtwThreadWorker>,
    // Store closures instead of calling `.forget()` which leaks
    _on_message: Box<Closure<dyn FnMut(MessageEvent)>>,
}

impl _Worker {
    fn new(wgs: WorkerGlobalScope) -> Self {
        let atw_thw = Rc::new(AtwThreadWorker::new(wgs));

        let on_message = Self::create_onmessage(atw_thw.clone());
        atw_thw.set_callback_of("onmessage", on_message.as_ref());

        Self {
            atw_thw,
            _on_message: Box::new(on_message),
        }
    }

    fn create_onmessage(atw_thw: Rc<AtwThreadWorker>) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::wrap(Box::new(move |me: MessageEvent| {
            let ref data = me.data();
            // debug_ln!("on_message(): data: {:?}", data);

            let (ref id, ref task_msg) = atw_decode_req_msg(data);
            Self::on_request_inner(atw_thw.clone(), id, task_msg);
        }) as Box<dyn FnMut(MessageEvent)>)
    }

    fn on_request_inner(atw_thw: Rc<AtwThreadWorker>, req_id: &str, task_msg: &JsValue) {
        // debug_ln!("on_request_inner(): req_id: {}", req_id);

        let (ref name, ref jsv) = decode_task_msg(task_msg);
        // debug_ln!("on_request_inner(): task: {}", name);

        // TODO - refactor with `enum` later
        match name.as_str() {
            "job-clos" | "job-aclos" => {
                type TypeT = Pin<Box<dyn Future<Output = Result<JsValue, JsValue>>>>;
                job::Job::<TypeT>::run(jsv, atw_thw, req_id);
            },
            "job-js" => job::run_job_js(jsv, atw_thw, req_id, false),
            "job-js-async" => job::run_job_js(jsv, atw_thw, req_id, true),
            _ => {
                let msg = format!("unknown task: {}", name);
                debug_ln!("err: {}", &msg);
                panic!("{}", msg);
            },
        }
    }
}
