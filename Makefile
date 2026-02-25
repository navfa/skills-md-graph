.PHONY: build test lint fmt check bench clean

build:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --check

check: fmt lint test

bench:
	cargo bench --bench bench_scan

clean:
	cargo clean

run:
	cargo run -- $(ARGS)
