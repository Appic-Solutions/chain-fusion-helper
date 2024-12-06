# Makefile

# Build target
build:
	@echo "Building Transaction Logeer Canister..."
	cargo build --release --target wasm32-unknown-unknown --package transaction_logger
	candid-extractor target/wasm32-unknown-unknown/release/transaction_logger.wasm > transaction_logger.did


