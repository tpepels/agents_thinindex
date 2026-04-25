.PHONY: build test install uninstall fmt clippy

build:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

install:
	./install.sh

uninstall:
	./uninstall.sh