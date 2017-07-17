ENV_VARS = PATH=/opt/local/bin:/opt/local/sbin:/Users/jessejones/.cargo/bin/:/usr/local/git/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin RUST_BACKTRACE=1

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
	 @$(ENV_VARS) cargo run -q --example=battle_bots -- --log-level=debug --no-colors --max-time=200s

.PHONY: telephone
telephone:
	@$(ENV_VARS) cargo run -q --example=telephone -- --log-level=debug --no-colors --max-time=20s

# This will update minor version numbers.
# To upate a major version number you need to edit the cargo file.
.PHONY: update
update:
	@cargo update

.PHONY: clean
clean:
	@rm -rf target
