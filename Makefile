SHELL := /bin/bash

CONFIG_PATH=config.toml

deploy-contract:
	@chmod +x ./contracts/deploy.sh
	@./contracts/deploy.sh
	@cp contracts/out/ZkAuction.sol/ZkAuction.json assets/ZkAuction.json

test-submit-proof:
	cd sp1-prover && make elf-commit
	RUST_BACKTRACE=1 cargo test --release --color=always --lib tests::test_submit_proof --no-fail-fast --manifest-path prover-sdk/Cargo.toml -- --exact -Z unstable-options --show-output --nocapture

deposit-to-aligned:
	aligned deposit-to-batcher \
    --rpc_url https://ethereum-holesky-rpc.publicnode.com \
    --network holesky \
    --keystore_path $(KEYSTORE_PATH) \
    --amount $(AMOUNT)ether

test-prove:
	cd sp1-prover && make gen-key && make elf-commit
	cargo test --release --color=always --lib tests::test_sp1_prover --no-fail-fast --manifest-path prover-sdk/Cargo.toml -- --exact -Z unstable-options --show-output

update-abi:
	cd contracts && rm -rf cache out broadcast && forge build
	cp contracts/out/ZkAuction.sol/ZkAuction.json assets/ZkAuction.json

taplo-fmt:
	taplo format --config taplo/taplo.toml