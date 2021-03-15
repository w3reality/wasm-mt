all: readme test build

readme:
	make -f readme.mk
	make -C crates/pool -f ../../readme.mk

test: test-mt test-mt-units test-mt-examples \
	test-pool test-pool-units test-pool-examples

# 'test' or 'ci'
TARGET ?= test
ci:
	TARGET=ci make test

test-mt:
	make -C tests/crates $(TARGET)
test-mt-units:
	make -f units.mk $(TARGET)
test-mt-examples:
	make -C examples/parallel $(TARGET)
	make -C examples/fib $(TARGET)
	make -C examples/arraybuffers $(TARGET)

test-pool:
	make -C crates/pool/tests/crates $(TARGET)
test-pool-units:
	make -C crates/pool -f units.mk $(TARGET)
test-pool-examples:
	make -C crates/pool/examples/http $(TARGET)
	make -C crates/pool/examples/pool_arraybuffers $(TARGET)


build: build-mt-examples build-pool-examples
build-mt-examples:
	make -C examples/exec
	make -C examples/executors
	make -C examples/parallel
	make -C examples/fib
	make -C examples/arraybuffers
build-pool-examples:
	make -C crates/pool/examples/pool_exec
	make -C crates/pool/examples/http
	make -C crates/pool/examples/pool_arraybuffers

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

