.PHONY: run dev bundle app dist changelog release setup-releasefmt fmt-check lint test check

run: ## Run the app (release build)
	cargo run --release

dev: ## Run the app (debug build, verbose logs)
	RUST_LOG=knoa=debug cargo run

bundle: ## Build target/release/Knoa.app with its icon and Sparkle (needs Xcode)
	scripts/bundle.sh

app: bundle ## Build and open Knoa.app
	open target/release/Knoa.app

dist: ## Signed, notarized Knoa.dmg and update zip in target/dist, on this Mac
	KNOA_SIGN_IDENTITY="$${KNOA_SIGN_IDENTITY:-Developer ID Application}" \
		KNOA_NOTARY_PROFILE="$${KNOA_NOTARY_PROFILE:-knoa}" scripts/package.sh

setup-release: ## Once: store the signing certificate and keys for the Release workflow
	scripts/setup-release.sh

changelog: ## What the next release's notes will say (needs git-cliff)
	git-cliff --unreleased --strip all

release: ## Release VERSION (like 0.2.0) through the Release workflow on GitHub
	@test -n "$(VERSION)" || { echo "usage: make release VERSION=0.2.0"; exit 1; }
	gh workflow run release.yml --ref main -f version=$(VERSION)
	@sleep 5
	gh run watch "$$(gh run list --workflow release.yml --limit 1 --json databaseId -q '.[0].databaseId')"

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
