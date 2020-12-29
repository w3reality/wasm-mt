all: wasm

# WASM_PACK_BUILD := RUSTFLAGS="-Z macro-backtrace" wasm-pack build --dev
WASM_PACK_BUILD := wasm-pack build --release

wasm:
	$(WASM_PACK_BUILD) --target no-modules

test:
	WASM_BINDGEN_TEST_TIMEOUT=60 wasm-pack test --headless --chrome
ci:
	wasm-pack test --headless --chrome --release

clean:
	rm -rf ./pkg/*
