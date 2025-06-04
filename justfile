build:
	anchor build --no-idl
	cargo build --workspace

localnet:
	solana-test-validator -q --reset &
	sleep 3
	anchor deploy

lint:
	cargo fmt --all
	cargo clippy --workspace -- -D warnings