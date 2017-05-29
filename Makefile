.PHONY: build
build:
	@cargo build

.PHONY: test
test:
	@cargo test --color=never

.PHONY: run
run:
	@cargo run

.PHONY: battle_bots
battle_bots:
	@RUST_BACKTRACE=1 cargo run --example=battle_bots -- --no-colors

# This will update minor version numbers.
# To upate a major version number you need to edit the cargo file.
.PHONY: update
update:
	@cargo update

.PHONY: clean
clean:
	@rm -rf target
