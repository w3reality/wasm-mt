export default async function (name) {
    const pkgJs = await (await fetch(`./pkg/${name}.js`)).text();

    // Create the 'pure' version of the wasm_bindgen's `init()`
    const initJs = `return () => { ${pkgJs} return wasm_bindgen; };`;
    const init = (new Function(initJs)).call(null);

    const wbg = init();
    const wasm = await wbg(`./pkg/${name}_bg.wasm`);
    // console.log('wasm:', wasm);
    wbg.app();

    return wbg;
}
