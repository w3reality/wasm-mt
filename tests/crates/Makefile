all: test

#test: test-node test-web
test: test-web

test-node:
	wasm-pack test --node ./node
test-web:
	wasm-pack test --headless --chrome ./web  # `--chrome`, `--firefox`, or `--safari`
test-web-no-headless: # interactive with the fixed port number 8000
	wasm-pack test --chrome ./web

.PHONY: babel test
