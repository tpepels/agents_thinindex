.PHONY: build test install uninstall fmt clippy license-audit ci-check package-release release-check

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

ci-check:
	scripts/check-ci

package-release:
	scripts/package-release

release-check:
	scripts/check-release

install:
	./install.sh

uninstall:
	./uninstall.sh
