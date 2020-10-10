all:
	cargo build

test:
	cargo test
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt -- --check

clean:
	cargo clean
