all: readme test build

readme: header.md src/lib.rs
	cat header.md > README.md
	cat src/lib.rs | grep '^//!' | sed -E 's/\/\/! ?//g' >> README.md
test: test-mt test-mt-units test-mt-examples \
	test-pool test-pool-units test-pool-examples

test-mt:
	make -C tests/crates
test-mt-units:
	wasm-pack test . --headless --firefox -- --lib  # https://rustwasm.github.io/wasm-pack/book/commands/test.html
test-mt-examples:
	make -C examples/parallel test
	make -C examples/fib test

test-pool:
	make -C crates/pool/tests/crates
test-pool-units:
	wasm-pack test crates/pool --headless --firefox -- --lib
test-pool-examples:
	make -C crates/pool/examples/http test


build: build-mt-examples build-pool-examples
build-mt-examples:
	make -C examples/exec
	make -C examples/executors
	make -C examples/parallel
	make -C examples/fib
build-pool-examples:
	make -C crates/pool/examples/pool_exec
	make -C crates/pool/examples/http

doc: doc-mt doc-mt-test doc-pool doc-pool-test
doc-mt:
	cargo doc --no-deps
doc-mt-dev:
	#cargo watch -x 'doc --no-deps'
	cargo watch -s 'make doc-mt && say ok || say error'
doc-mt-test:
	cargo doc --no-deps --manifest-path crates/test/Cargo.toml
doc-pool:
	cargo doc --no-deps --manifest-path crates/pool/Cargo.toml
doc-pool-test:
	cargo doc --no-deps --manifest-path crates/pool/crates/test/Cargo.toml

