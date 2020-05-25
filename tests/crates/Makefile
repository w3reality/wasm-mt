all: test

#test: test-node test-web
test: test-web

test-node:
	wasm-pack test --node ./node
# --chrome  # useless for 'fib'; seeing [SEVERE]: Timed out receiving message from renderer: 0.100; USE firefox for the moment....
test-web:
	wasm-pack test --headless --firefox ./web
test-web-no-headless: # interactive with the fixed port number 8000
	wasm-pack test --chrome ./web

.PHONY: babel test
