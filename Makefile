SHELL := /bin/bash

CONFIG_PATH=config.toml

start-cli:
	@cargo install --bin tahken --path .
	@tahken \
		--config-path $(CONFIG_PATH) \
		--keystore-path $(KEYSTORE_PATH)
	@rm -rf ~/.tahken/keystores/wallet_tahken
	@mkdir -p ~/.tahken/keystores
	@cp $(KEYSTORE_PATH) ~/.tahken/keystores


deploy-contract:
	@. ./contracts/deploy.sh