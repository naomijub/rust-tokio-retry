test:
	cargo test --all-features

clippy:
	cargo clippy --all-features -- -W clippy::all -W clippy::nursery -D warnings

fmt:
	cargo fmt --all

lint: fmt clippy

all: fmt clippy test

pedantic:
	cargo clippy --all-features -- -W clippy::pedantic -D warnings