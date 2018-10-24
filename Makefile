all: test

doc:
	@cargo doc

test: cargotest

cargotest:
	@cargo test

format-check:
	@rustup component add rustfmt-preview 2> /dev/null
	@cargo fmt -- --check

.PHONY: all doc test cargotest format-check
