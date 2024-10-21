SHELL := /bin/bash

CONFIG_PATH=config.toml

start-cli:
	@cargo install --path .
	@tahm-kench-cli \
		--config-path $(CONFIG_PATH) \
		--keystore-path $(KEYSTORE_PATH)
	@rm -rf ~/.zk_auction/keystores/wallet_zk_auction
	@mkdir -p ~/.zk_auction/keystores
	@cp $(KEYSTORE_PATH) ~/.zk_auction/keystores


deploy-contract:
	@. ./contracts/deploy.sh