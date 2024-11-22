SHELL := /bin/bash

CONFIG_PATH=config.toml

install:
	cargo install --path crates/cli --force

deploy-contract:
	@chmod +x ./crates/cli/contracts/deploy.sh
	@./crates/cli/contracts/deploy.sh
	@cp crates/cli/contracts/out/ZkAuction.sol/ZkAuction.json crates/cli/assets/ZkAuction.json

deposit-to-aligned:
	aligned deposit-to-batcher \
    --rpc_url https://ethereum-holesky-rpc.publicnode.com \
    --network holesky \
    --keystore_path $(KEYSTORE_PATH) \
    --amount $(AMOUNT)ether

update-abi:
	cd crates/cli/contracts && rm -rf cache out broadcast && forge build
	cp crates/cli/contracts/out/ZkAuction.sol/ZkAuction.json crates/cli/assets/ZkAuction.json

taplo-fmt:
	taplo format --config taplo/taplo.toml

test-find-winner:
	cd crates/sp1-prover && make gen-key && make elf-vk
	RUST_BACKTRACE=1 cargo test --release --color=always --lib tests::test_find_winner \
	--no-fail-fast --manifest-path crates/prover-sdk/Cargo.toml -- --exact -Z unstable-options --show-output --nocapture

test-mint:
	cargo test --color=always --lib tests::test_auction::test::test_mint \
	--no-fail-fast --manifest-path crates/cli/Cargo.toml -- --exact -Z unstable-options --show-output --nocapture

test-flow:
	RUST_BACKTRACE=1 cargo test --release --color=always --lib tests::test_auction::test::test_auction_service \
	--no-fail-fast --manifest-path crates/cli/Cargo.toml -- --exact -Z unstable-options --show-output --nocapture
