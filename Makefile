SHELL := /bin/bash

CONFIG_PATH=config.toml

start-cli:
	@cargo install --bin tahken --path . --force
	@tahken \
		--config-path $(CONFIG_PATH) \
		--keystore-path $(KEYSTORE_PATH)
	@rm -rf ~/.tahken/keystores/wallet_tahken
	@mkdir -p ~/.tahken/keystores
	@cp $(KEYSTORE_PATH) ~/.tahken/keystores/wallet_tahken

deploy-contract:
	@chmod +x ./contracts/deploy.sh
	@./contracts/deploy.sh
	@cp contracts/out/ZkAuction.sol/ZkAuction.json assets/ZkAuction.json

test-auction:
	RUST_BACKTRACE=1 cargo test --release --color=always --message-format=json-diagnostic-rendered-ansi --no-run --package zk_auction --lib tests::test_auction_service::test_auction_service --profile dev
	/home/ubuntu/.cargo/bin/cargo test --color=always --message-format=json-diagnostic-rendered-ansi --no-run --package zk_auction --lib tests::test_auction_service::test_auction_service

test-submit-proof:
	RUST_BACKTRACE=1 cargo test --release --color=always --lib tests::test_submit_proof --no-fail-fast --manifest-path /home/ubuntu/code/zkp/tahm-kench/prover-sdk/Cargo.toml -- --exact -Z unstable-options --show-output --nocapture

deposit-to-aligned:
	aligned deposit-to-batcher \
    --rpc_url https://ethereum-holesky-rpc.publicnode.com \
    --network holesky \
    --keystore_path $(KEYSTORE_PATH) \
    --amount 0.004ether
