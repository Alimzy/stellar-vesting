.PHONY: test fmt lint check build size clean

test:
	cargo test --locked

fmt:
	cargo fmt --all

lint:
	cargo clippy --locked --all-targets -- -D warnings

check:
	cargo fmt --all -- --check
	$(MAKE) lint
	$(MAKE) test

build:
	cargo build --locked --release --target wasm32v1-none -p vesting

size: build
	@ls -l target/wasm32v1-none/release/vesting.wasm | awk '{print $$5 " bytes"}'

clean:
	cargo clean
