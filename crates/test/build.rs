use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=js/babel.min.js");

    fs::create_dir("pkg").unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });

    echo("*", &Path::new(&format!("pkg/.gitignore"))).unwrap();

    let mut js = String::new();
    js.push_str("const exports = {}, module = 42;\n"); // SHIM: `module` should be defined to something!
    js.push_str(&cat(&Path::new("js/babel.min.js")).unwrap());
    js.push_str("export function transform(input, config) { return exports.transform(input, config); }\n");
    echo(&js, &Path::new(&format!("pkg/babel-transform.js"))).unwrap();
}

// Using `cat` and `echo` from
// https://doc.rust-lang.org/rust-by-example/std_misc/fs.html

fn cat(path: &Path) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;

    Ok(s)
}

fn echo(s: &str, path: &Path) -> io::Result<()> {
    let mut f = File::create(path)?;

    f.write_all(s.as_bytes())
}
