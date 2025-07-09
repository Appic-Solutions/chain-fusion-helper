# Makefile

# Build target
build_transaction_logger:
	@echo "Building Transaction Logeer Canister..."
	cargo build --release --target wasm32-unknown-unknown --package transaction_logger
	candid-extractor target/wasm32-unknown-unknown/release/transaction_logger.wasm > transaction_logger.did

# Build Proxy canister
build_proxy_canister:
	@echo "Building Proxy canister"
	cargo build --release --target wasm32-unknown-unknown --package proxy_canister
	candid-extractor target/wasm32-unknown-unknown/release/proxy_canister.wasm > proxy_canister.did

