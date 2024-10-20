SHELL := /bin/bash

cargo-run:
	@cargo run --release
	@cargo install --path .
deploy-contract:
	@. ./contracts/.env && . ./contracts/deploy.sh