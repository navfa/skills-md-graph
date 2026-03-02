.PHONY: build test lint fmt check bench clean install scan graph lint-skills query export

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

install:
	cargo install --path .

run:
	cargo run -- $(ARGS)

scan:
	cargo run -- scan $(path)

graph:
	cargo run -- graph $(path) $(ARGS)

lint-skills:
	cargo run -- lint $(path)

query:
	cargo run -- query $(path) $(ARGS)

export:
	cargo run -- export $(path) $(ARGS)
