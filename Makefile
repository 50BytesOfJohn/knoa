.PHONY: run dev fmt fmt-check lint test check

run: ## Run the app (release build)
	cargo run --release

dev: ## Run the app (debug build, verbose logs)
	RUST_LOG=knoa=debug cargo run

fmt: ## Format Rust and TOML
	cargo fmt --all
	taplo fmt

fmt-check: ## Verify formatting without changing files
	cargo fmt-check
	taplo fmt --check

lint: ## Clippy with warnings as errors
	cargo lint

test: ## Run all tests
	cargo test --workspace --locked

check: fmt-check lint test ## Everything CI runs
