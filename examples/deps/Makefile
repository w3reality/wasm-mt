all: wasm

# WASM_PACK_BUILD := RUSTFLAGS="-Z macro-backtrace" wasm-pack build --dev
WASM_PACK_BUILD := wasm-pack build --release

wasm:
	$(WASM_PACK_BUILD) --target no-modules

test:
	wasm-pack test --headless --firefox

clean:
	rm -rf ./pkg/*
