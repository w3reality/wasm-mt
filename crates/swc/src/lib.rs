// CommonJs transform utility originally based on
// https://github.com/swc-project/swc/blob/edf74fc1/wasm/src/lib.rs

use once_cell::sync::Lazy;
use std::{
    fmt::{self, Display, Formatter},
    io::{self, Write},
    sync::{Arc, RwLock},
};

use swc::{
    config::{Config, ModuleConfig, JscConfig, Options},
    Compiler,
};
use swc_common::{
    errors::{DiagnosticBuilder, Emitter, Handler, SourceMapperDyn},
    FileName, FilePathMapping, SourceMap,
    Globals, GLOBALS,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_transforms_module::common_js;

pub fn transform_sync(input: &str) -> Option<String> {
    let opts = Options {
        config: Config {
            jsc: JscConfig {
                target: Some(EsVersion::Es2017), // Do keep async/await
                ..Default::default()
            },
            module: Some(ModuleConfig::CommonJs(common_js::Config {
                ..Default::default()
            })),
            ..Default::default()
        },
        swcrc: false,
        ..Default::default()
    };

    let (c, handler, _errors) = compiler();
    let fm = c.cm.new_source_file(FileName::Anon, input.into());

    let mut out = None;
    GLOBALS.set(&Globals::new(), || {
        out = c.process_js_file(fm, &handler, &opts).ok().map(|x| x.code);
    });

    out
}

fn compiler() -> (Compiler, Arc<Handler>, BufferedError) {
    let cm = codemap();
    let (handler, errors) = new_handler(cm.clone());
    let c = Compiler::new(cm.clone());

    (c, handler, errors)
}

/// Get global sourcemap
fn codemap() -> Arc<SourceMap> {
    static CM: Lazy<Arc<SourceMap>> =
        Lazy::new(|| Arc::new(SourceMap::new(FilePathMapping::empty())));

    CM.clone()
}

/// Creates a new handler which emits to returned buffer.
fn new_handler(_cm: Arc<SourceMapperDyn>) -> (Arc<Handler>, BufferedError) {
    let e = BufferedError::default();

    let handler = Handler::with_emitter(true, false, Box::new(MyEmiter::default()));

    (Arc::new(handler), e)
}
#[derive(Clone, Default)]
struct MyEmiter(BufferedError);

impl Emitter for MyEmiter {
    fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
        let z = &(self.0).0;
        for msg in &db.message {
            z.write().unwrap().push_str(&msg.0);
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct BufferedError(Arc<RwLock<String>>);

impl Write for BufferedError {
    fn write(&mut self, d: &[u8]) -> io::Result<usize> {
        self.0
            .write()
            .unwrap()
            .push_str(&String::from_utf8_lossy(d));

        Ok(d.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Display for BufferedError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0.read().unwrap(), f)
    }
}