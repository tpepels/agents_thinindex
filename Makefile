.PHONY: build test install uninstall fmt clippy license-audit

build:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

license-audit:
	cargo deny check licenses

install:
	./install.sh

uninstall:
	./uninstall.sh
