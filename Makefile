.PHONY: run dev bundle app dist changelog release fmt fmt-check lint test check

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
run: ## Run the app (release build)
	$(COOL) cargo build --release
	cargo run --release

# Running the bundle's executable directly keeps the logs here and Ctrl-C quits the app.
dev: ## Run Knoa Dev: the debug build as an app bundle, verbose logs (needs Xcode)
	$(COOL) env KNOA_DEV=1 scripts/bundle.sh
	RUST_LOG=knoa=debug target/debug/Knoa.app/Contents/MacOS/Knoa

bundle: ## Build target/release/Knoa.app with its icon and Sparkle (needs Xcode)
	$(COOL) scripts/bundle.sh

app: bundle ## Build and open Knoa.app
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

check: fmt-check lint test ## Everything CI runs
