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
	@PATH=/opt/local/bin:/opt/local/sbin:/opt/local/bin:/opt/local/sbin:/opt/local/bin:/opt/local/sbin:/Users/jessejones/.cargo/bin/:/usr/local/git/bin:/Users/jessejones/Library/Haskell/bin:/opt/local/bin:/opt/local/sbin:/opt/local/bin:/opt/local/sbin:/opt/local/bin:/opt/local/sbin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin RUST_BACKTRACE=1 cargo run -q --example=battle_bots -- --log-level=debug --no-colors --max-time=20s

# This will update minor version numbers.
# To upate a major version number you need to edit the cargo file.
.PHONY: update
update:
	@cargo update

.PHONY: clean
clean:
	@rm -rf target
