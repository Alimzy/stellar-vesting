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

MAX_WASM_BYTES ?= 23000

size: build
	@f=target/wasm32v1-none/release/vesting.wasm; \
	size=$$(wc -c < "$$f" | tr -d '[:space:]'); \
	echo "size: $$size bytes (budget: $(MAX_WASM_BYTES) bytes)"; \
	if [ "$$size" -gt "$(MAX_WASM_BYTES)" ]; then \
		echo "ERROR: WASM size $$size bytes exceeds budget of $(MAX_WASM_BYTES) bytes"; \
		exit 1; \
	fi

clean:
	cargo clean
