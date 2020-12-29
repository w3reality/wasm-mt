all: test

# https://rustwasm.github.io/wasm-pack/book/commands/test.html

test:
	wasm-pack test . --headless --chrome -- --lib
ci:
	wasm-pack test . --headless --chrome --release -- --lib
