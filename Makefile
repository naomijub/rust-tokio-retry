test:
	cargo test --all-features

typos:
	typos

clippy:
	cargo clippy --all-features -- -W clippy::all -W clippy::nursery -D warnings

fmt:
	cargo fmt --all

lint: typos fmt clippy

all: typos fmt clippy test

pedantic:
	cargo clippy --all-features -- -W clippy::pedantic -D warnings