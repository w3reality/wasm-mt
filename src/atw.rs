// rust-wasm porting of -- https://github.com/w3reality/async-thread-worker

use crate::{debug_ln, console_ln};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Array, Function, Object, Promise, Reflect};
use web_sys::{MessageEvent, Worker, WorkerGlobalScope};
use uuid::Uuid;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

fn atw_encode_req_msg(id: &Uuid, payload: &JsValue) -> Object {
    let msg = Object::new();
    Reflect::set(msg.as_ref(), &JsValue::from("id"), &JsValue::from(&id.to_string())).unwrap();
    Reflect::set(msg.as_ref(), &JsValue::from("payload"), payload).unwrap();
    msg
}

pub fn atw_decode_req_msg(msg: &JsValue) -> (String, JsValue) {
    let id = Reflect::get(msg, &JsValue::from("id"))
        .unwrap_throw().as_string().unwrap_throw();
    let payload = Reflect::get(msg, &JsValue::from("payload"))
        .unwrap_throw();
    (id, payload)
}

fn atw_encode_result_msg(id: &str, result: &JsValue, is_ok: bool) -> Object {
    let msg = Object::new();
    Reflect::set(msg.as_ref(), &JsValue::from("id"), &JsValue::from(id)).unwrap();
    Reflect::set(msg.as_ref(), &JsValue::from("result"), result).unwrap();
    Reflect::set(msg.as_ref(), &JsValue::from("isOk"), &JsValue::from(is_ok)).unwrap();
    msg
}

fn atw_decode_result_msg(msg: &JsValue) -> (Uuid, JsValue, bool) {
    let id = Reflect::get(msg, &JsValue::from("id"))
        .unwrap_throw().as_string().unwrap_throw();
    let id = Uuid::parse_str(&id).unwrap_throw();

    let result = Reflect::get(msg, &JsValue::from("result"))
        .unwrap_throw();
    let is_ok = Reflect::get(msg, &JsValue::from("isOk"))
        .unwrap_throw().as_bool().unwrap_throw();
    (id, result, is_ok)
}

// Bindings such as `post_message_with_transfer()` seem not available
// in `web_sys::WorkerGlobalScope` (as opposed to `web_sys::Worker`).
// So, we define and use a custom binding `JsWgs` instead.

pub struct ThreadWorker {
    wgs: JsWgs,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = _)]
    type JsWgs;

    #[wasm_bindgen(method, js_name = postMessage)]
    fn post_message_with_transfer(this: &JsWgs, data: &JsValue, transfer: &Array);
}

impl JsWgs {
    fn new(wgs: WorkerGlobalScope) -> Self {
        wgs.unchecked_into::<JsWgs>()
    }
}

impl ThreadWorker {
    pub fn new(wgs: WorkerGlobalScope) -> Self {
        Self {
            wgs: JsWgs::new(wgs),
        }
    }

    pub fn send_response(&self, req_id: &str, payload: &JsValue, transfer: Option<&Array>) {
        debug_ln!("send_response(): req_id: {} payload: {:?} transfer: {:?}", req_id, payload, transfer);

        let default = Array::new();
        let transfer = transfer.unwrap_or(&default);
        self.wgs.post_message_with_transfer(
            &atw_encode_result_msg(req_id, payload, true), transfer);
    }

    pub fn send_error(&self, req_id: &str, error: &JsValue) {
        debug_ln!("send_error(): req_id: {} error: {:?}", req_id, error);

        self.wgs.post_message_with_transfer(
            &atw_encode_result_msg(req_id, error, false), &Array::new());
    }

    pub fn set_callback_of(&self, target: &str, cb: &JsValue) {
        // debug_ln!("set_callback_of(): target: {}", target);
        Reflect::set(&self.wgs, &JsValue::from(target),
            &cb.unchecked_ref::<Function>().to_owned()).unwrap_throw();
    }
}

type RrMap = HashMap<Uuid, (Function, Function)>;

pub struct Thread {
    worker: Worker,
    _on_message: Box<Closure<dyn FnMut(MessageEvent)>>,
    _on_error: Box<Closure<dyn FnMut(MessageEvent)>>,
    rr_map: Rc<RefCell<RrMap>>,
    is_terminated: RefCell<bool>,
}

impl Thread {
    pub fn new(script_url: &str) -> Self {
        let worker = Worker::new(script_url);
        if let Err(ref jsv) = worker {
            console_ln!("error: {:?}", jsv);

            // https://developer.mozilla.org/en-US/docs/Web/API/Worker
            // https://bugs.webkit.org/show_bug.cgi?id=22723
            // https://wpt.fyi/results/workers/semantics/multiple-workers/003.html
            console_ln!("Hint: On Safari, nested Web Workers might not be supported as of now.");
        }
        let worker = worker.unwrap_throw();

        let rr_map = Rc::new(RefCell::new(HashMap::new()));
        let on_message = Self::create_onmessage(rr_map.clone());
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref::<Function>()));
        let on_error = Self::create_onerror(rr_map.clone());
        worker.set_onerror(Some(on_error.as_ref().unchecked_ref::<Function>()));

        Self {
            worker,
            rr_map,
            _on_message: Box::new(on_message),
            _on_error: Box::new(on_error),
            is_terminated: RefCell::new(false),
        }
    }

    fn create_onmessage(rr_map: Rc<RefCell<RrMap>>) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::wrap(Box::new(move |me: MessageEvent| {
            let msg = me.data();

            // debug_ln!("on_message(): msg: {:?}", &msg);
            if msg == JsValue::NULL {
                debug_ln!("on_message(): msg: {:?}; oops, `.await` will hang!!", msg);
                return;
            }

            let (id, result, is_ok) = atw_decode_result_msg(&msg);

            let mut rr_map = rr_map.borrow_mut();
            assert!(rr_map.get(&id).is_some());
            let (res, rej) = rr_map.remove(&id).unwrap_throw();
            assert!(rr_map.get(&id).is_none());

            (if is_ok { res } else { rej })
                .call1(&JsValue::NULL, &result)
                .unwrap_throw();
        }) as Box<dyn FnMut(MessageEvent)>)
    }

    fn create_onerror(rr_map: Rc<RefCell<RrMap>>) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::wrap(Box::new(move |_me: MessageEvent| {
            console_ln!("terminated by error");
            let mut rr_map = rr_map.borrow_mut();
            let cancels = rr_map.len();
            debug_ln!("cancel_pending_requests(): canceling {} pending reqs", cancels);
            for (req_id, (_res, rej)) in rr_map.drain() {
                debug_ln!("canceling req: {}", &req_id);
                rej.call1(&JsValue::NULL,
                    &JsValue::from(&format!("Thread: req[{}] canceled", &req_id))).unwrap();
            }
        }) as Box<dyn FnMut(MessageEvent)>)
    }

    fn new_req_id(rr_map: Ref<RrMap>) -> Uuid {
        let mut collision_count = 0;
        loop {
            let uuid = Uuid::new_v4();
            if rr_map.get(&uuid).is_none() {
                break uuid;
            } else {
                debug_ln!("oops: unlikely collision!!");
                collision_count += 1;
                if collision_count > 4 {
                    panic!("too many uuid collisions");
                }
            }
        }
    }

    pub async fn send_request(&self, payload: &JsValue, transfer: Option<&Array>) -> Result<JsValue, JsValue> {
        let promise = Promise::new(&mut |res, rej| {
            if *self.is_terminated.borrow() {
                rej.call1(&JsValue::NULL, &JsValue::from("worker already terminated")).unwrap_throw();
                return;
            }

            let req_id = Self::new_req_id(self.rr_map.borrow());
            self.rr_map.borrow_mut().insert(req_id, (res, rej));

            let default = Array::new();
            let transfer = transfer.unwrap_or(&default);
            self.worker.post_message_with_transfer(
                &atw_encode_req_msg(&req_id, payload), transfer).unwrap_throw();
        });

        JsFuture::from(promise).await
    }

    fn cancel_pending_requests(&self) {
        let mut rr_map = self.rr_map.borrow_mut();

        let cancels = rr_map.len();
        debug_ln!("cancel_pending_requests(): canceling {} pending reqs", cancels);
        for (req_id, (_res, rej)) in rr_map.drain() {
            debug_ln!("canceling req: {}", &req_id);
            rej.call1(&JsValue::NULL,
                &JsValue::from(&format!("Thread: req[{}] canceled", &req_id))).unwrap();
        }
    }

    pub fn terminate(&self) {
        if *self.is_terminated.borrow() {
            debug_ln!("Thread::terminate(): nop; already terminated");
        } else {
            self.is_terminated.replace(true);
            self.cancel_pending_requests();
            self.worker.terminate();
        }
    }

    pub fn is_terminated(&self) -> bool {
        *self.is_terminated.borrow()
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        debug_ln!("Thread::drop(): called");
        self.terminate();
    }
}
