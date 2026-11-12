.PHONY: run dev dev-local bundle app dist changelog release fmt fmt-check lint test check check-wasm check-sdk

# Settings for this machine only, like FAST = 1. Git ignores the file.
-include local.mk

# Builds run at background priority, which keeps them on the efficiency cores so the Mac
# stays cool and the fans stay quiet. They take longer. FAST=1 builds at full speed.
ifeq ($(FAST),)
COOL := taskpolicy -c background
ifndef CARGO_BUILD_JOBS
# One job per efficiency core. Intel Macs have none, so they get half their cores.
CARGO_BUILD_JOBS := $(shell sysctl -n hw.perflevel1.logicalcpu 2>/dev/null || \
	echo $$(( $$(sysctl -n hw.logicalcpu) / 2 )))
endif
export CARGO_BUILD_JOBS
endif

# The app itself runs at normal priority, so only the build step goes through $(COOL).
run: ## Run Knoa Dev against staging (optimized, without a bundle)
	$(COOL) env KNOA_CHANNEL=dev cargo build --release
	KNOA_CHANNEL=dev cargo run --release

# Running the bundle's executable directly keeps the logs here and Ctrl-C quits the app.
dev: ## Run Knoa Dev against staging (debug bundle, verbose logs; needs Xcode)
	$(COOL) env KNOA_CHANNEL=dev KNOA_DEV=1 scripts/bundle.sh
	RUST_LOG=knoa=debug target/debug/Knoa.app/Contents/MacOS/Knoa

dev-local: ## Run Knoa Dev against the local API, with staging Clerk
	KNOA_API_URL=http://localhost:8787 $(MAKE) dev

bundle: ## Build optimized Knoa Dev in target/release/Knoa.app (needs Xcode)
	$(COOL) env KNOA_CHANNEL=dev scripts/bundle.sh

app: bundle ## Build and open the optimized Knoa Dev bundle
	open target/release/Knoa.app

dist: ## Signed, notarized Knoa.dmg and update zip in target/dist (see scripts/package.sh)
	$(COOL) scripts/package.sh

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
	$(COOL) cargo lint

test: ## Run all tests
	$(COOL) cargo test --workspace --locked

check-wasm: ## Check the crates used by the API Worker for WebAssembly
	$(COOL) cargo check --target wasm32-unknown-unknown -p knoa-core -p knoa-fetch -p knoa-ai --locked

check-sdk: ## Test and typecheck the shared TypeScript package (pnpm install in sdk/ first)
	cd sdk && pnpm test && pnpm typecheck

check: fmt-check lint test check-wasm check-sdk ## Format, lint, tests, WebAssembly and SDK
