# My Finances API - developer commands. Run `make` or `make help` to list them.
# Commands read DATABASE_URL and friends from .env; override per command with
# `DATABASE_URL=... make <target>`.

SHELL := /bin/bash
.DEFAULT_GOAL := help

CARGO  ?= cargo
SQLX   ?= sqlx
VERCEL ?= npx -y vercel@latest

##@ Help

.PHONY: help
help: ## Show this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage: make \033[36m<target>\033[0m\n"} \
		/^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 } \
		/^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) }' $(MAKEFILE_LIST)

##@ Setup

.PHONY: setup
setup: install-tools env db-setup ## First-time setup: tools, .env, database (create + migrate + seed)

.PHONY: install-tools
install-tools: ## Install rustfmt, clippy, sqlx-cli, cargo-deny, cargo-audit, cargo-watch
	rustup component add rustfmt clippy
	$(CARGO) install sqlx-cli --no-default-features --features rustls,postgres --locked
	$(CARGO) install cargo-deny cargo-audit cargo-watch --locked

.PHONY: env
env: ## Create .env from .env.example (if missing)
	@if [ -f .env ]; then echo ".env already exists"; else cp .env.example .env && echo "Created .env - fill in your values"; fi

##@ Development

.PHONY: dev
dev: ## Run the API locally on PORT (default 3001)
	$(CARGO) run

.PHONY: watch
watch: ## Run the API and restart on changes (cargo-watch)
	$(CARGO) watch -x run

.PHONY: run-release
run-release: ## Run the API locally with the optimized release build
	$(CARGO) run --release --bin local

.PHONY: docs
docs: ## Serve the Swagger UI for the local API on http://localhost:3002 (run `make dev` too)
	@echo "API docs: http://localhost:3002 (API expected on http://localhost:3001)"
	python3 -m http.server 3002 --bind 127.0.0.1 --directory postman

.PHONY: check
check: ## Type-check all targets
	$(CARGO) check --all-targets

##@ Build

.PHONY: build
build: ## Debug build of all targets
	$(CARGO) build --all-targets

.PHONY: build-release
build-release: ## Release build of the Vercel function binary
	$(CARGO) build --release --bin main

.PHONY: build-offline
build-offline: ## Build without a database using .sqlx/ (what Vercel and CI do)
	SQLX_OFFLINE=true $(CARGO) build --all-targets

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean

##@ Quality

.PHONY: fmt
fmt: ## Format the code
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Check formatting
	$(CARGO) fmt --all --check

.PHONY: lint
lint: ## Run clippy with warnings as errors
	$(CARGO) clippy --all-targets -- -D warnings

.PHONY: lint-fix
lint-fix: ## Apply clippy's automatic fixes
	$(CARGO) clippy --all-targets --fix --allow-dirty --allow-staged -- -D warnings

##@ Tests (need DATABASE_URL to a Postgres that allows CREATE DATABASE)

.PHONY: test
test: ## Run all tests
	$(CARGO) test

.PHONY: test-unit
test-unit: ## Run unit tests only (no database)
	$(CARGO) test --lib

.PHONY: test-integration
test-integration: ## Run integration tests only (tests/)
	$(CARGO) test --test '*'

.PHONY: test-one
test-one: ## Run tests matching a name: make test-one T=expense
	@test -n "$(T)" || (echo "Usage: make test-one T=<test name filter>" && exit 1)
	$(CARGO) test $(T) -- --nocapture

##@ Security

.PHONY: deny
deny: ## Check advisories, licenses, bans and sources (cargo-deny)
	$(CARGO) deny check advisories licenses bans sources

.PHONY: audit
audit: ## Check dependencies against the RustSec advisory database
	$(CARGO) audit

.PHONY: security
security: deny audit ## Run all dependency security checks

.PHONY: ci
ci: fmt-check lint test build-offline deny ## Run everything CI runs

##@ Database

.PHONY: db-setup
db-setup: db-create db-migrate db-seed ## Create, migrate and seed the database

.PHONY: db-create
db-create: ## Create the database in DATABASE_URL
	$(SQLX) database create

.PHONY: db-drop
db-drop: ## Drop the database in DATABASE_URL (asks for confirmation)
	$(SQLX) database drop

.PHONY: db-reset
db-reset: ## Drop, recreate and migrate the database, then seed (asks for confirmation)
	$(SQLX) database reset
	$(CARGO) run --bin seed

.PHONY: db-migrate
db-migrate: ## Apply pending migrations (use the direct connection, port 5432, for Supabase)
	$(SQLX) migrate run

.PHONY: db-revert
db-revert: ## Revert the last migration (only for reversible migrations)
	$(SQLX) migrate revert

.PHONY: db-status
db-status: ## Show applied and pending migrations
	$(SQLX) migrate info

.PHONY: db-new
db-new: ## Create a new migration: make db-new NAME=add_budget_notes
	@test -n "$(NAME)" || (echo "Usage: make db-new NAME=<migration_name>" && exit 1)
	$(SQLX) migrate add $(NAME)

.PHONY: db-seed
db-seed: ## Seed default years and categories (idempotent)
	$(CARGO) run --bin seed

.PHONY: db-clean
db-clean: ## Delete all data, keep the schema (refuses in production)
	$(CARGO) run --bin clean

.PHONY: sqlx-prepare
sqlx-prepare: ## Regenerate .sqlx/ offline query metadata (commit the result)
	$(CARGO) sqlx prepare -- --all-targets

.PHONY: sqlx-check
sqlx-check: ## Verify .sqlx/ is up to date with the queries
	$(CARGO) sqlx prepare --check -- --all-targets

##@ Vercel

.PHONY: vercel-link
vercel-link: ## Link this directory to a Vercel project
	$(VERCEL) link

.PHONY: vercel-dev
vercel-dev: ## Run the function locally through the Vercel CLI
	$(VERCEL) dev

.PHONY: deploy-preview
deploy-preview: ## Deploy a preview build
	$(VERCEL) deploy

.PHONY: deploy-prod
deploy-prod: ## Deploy to production
	$(VERCEL) deploy --prod

.PHONY: vercel-logs
vercel-logs: ## Show runtime logs for a deployment: make vercel-logs URL=<deployment-url>
	@test -n "$(URL)" || (echo "Usage: make vercel-logs URL=<deployment-url>" && exit 1)
	$(VERCEL) logs $(URL)

##@ Agent skills

.PHONY: skills-list
skills-list: ## List the project's agent skills (.agents/skills)
	npx -y skills list

.PHONY: skills-update
skills-update: ## Update the project's agent skills from skills-lock.json sources
	npx -y skills update -p -y
