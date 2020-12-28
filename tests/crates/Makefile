all: test

#test: test-node test-web
test: test-web

test-node:
	wasm-pack test --node ./node
test-web: # `--chrome`, `--firefox`, or `--safari`
	WASM_BINDGEN_TEST_TIMEOUT=60 wasm-pack test --headless --chrome ./web
test-web-release: # for CI
	wasm-pack test --headless --chrome ./web --release
test-web-no-headless: # interactive with the fixed port number 8000
	wasm-pack test --chrome ./web

.PHONY: test
