// CommonJs transform utility based on
// https://github.com/swc-project/swc/blob/master/wasm/src/lib.rs (edf74fc1 2020-12-20)

use once_cell::sync::Lazy;
use std::{
    fmt::{self, Display, Formatter},
    io::{self, Write},
    sync::{Arc, RwLock},
};
use wasm_mt_swc::prelude::{
    swc_common::{
        errors::{DiagnosticBuilder, Emitter, Handler, SourceMapperDyn},
        FileName, FilePathMapping, SourceMap,
    },
    swc_ecma_transforms::modules::common_js,
    // swc_ecma_parser::{Syntax, EsConfig},
};
use wasm_mt_swc::{
    config::{Config, ModuleConfig, JscConfig, JscTarget, Options},
    Compiler,
};

pub fn swc_transform(input: &str) -> Option<String> {
    let opts = Options {
        config: Some(Config {
            jsc: JscConfig {
                // This doesn't help; instead, using `import.meta` workaround in `create_ab_init()`
                // syntax: Some(Syntax::Es(EsConfig {
                //     import_meta: true,
                //     ..Default::default()
                // })),
                target: JscTarget::Es2017, // Do keep async/await
                ..Default::default()
            },
            module: Some(ModuleConfig::CommonJs(common_js::Config {
                ..Default::default()
            })),
            ..Default::default()
        }),
        swcrc: false,
        ..Default::default()
    };

    let (c, _errors) = compiler();
    let fm = c.cm.new_source_file(FileName::Anon, input.into());
    let out = c
        .process_js_file(fm, &Options {
            is_module: true,
            ..opts
        }).ok()?;

    Some(out.code.into())
}
fn compiler() -> (Compiler, BufferedError) {
    let cm = codemap();
    let (handler, errors) = new_handler(cm.clone());
    let c = Compiler::new(cm.clone(), handler);

    (c, errors)
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
