# Commander pod simulator: set up Rust, build, test and run the programs.
#
#   make setup      install Rust (rustup) if it's missing, then the pinned toolchain (rust/rust-toolchain.toml)
#   make build      build commander-sim, commander-practice and cardgen (release) into rust/target/release/
#   make test       run the full test suite
#   make env        print the lines that "activate" the environment: eval "$(make -s env)"
#   make shell      open a shell with the environment active (exit to leave)
#   make sim        a quick simulation:  make sim DECK=sauron POOL=t3 GAMES=200 AI=lookahead
#   make practice   practice mode in the browser (extra flags: ARGS="--port 9000")
#   make cardgen    run cardgen:  make cardgen ARGS="audit --mine"
#   make clean      remove the build output
#
# Builds, tests and runs go through a memory cap (systemd-run, 24G) where systemd is available, so a runaway can't
# take the machine down; elsewhere (macOS) they run directly.

SHELL := /bin/bash
.DEFAULT_GOAL := help

ROOT      := $(CURDIR)
RUST      := $(ROOT)/rust
BIN       := $(RUST)/target/release
CARGO_BIN := $(or $(CARGO_HOME),$(HOME)/.cargo)/bin
export PATH := $(CARGO_BIN):$(PATH)
export COMMANDER_SIM_ROOT := $(ROOT)

MEM ?= 24G
CAP := $(shell systemd-run --user --scope -q true >/dev/null 2>&1 && echo "systemd-run --user --scope -q -p MemoryMax=$(MEM) -p MemorySwapMax=0")

DECK    ?= sheoldred
POOL    ?= t3
PROFILE ?= loose
GAMES   ?= 100
AI      ?= adaptive
JOBS    ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
ARGS    ?=

.PHONY: help setup build test env shell sim practice cardgen clean

help:
	@sed -n '3,11p' $(firstword $(MAKEFILE_LIST)) | sed 's/^# \{0,3\}//'

setup:
	@if ! command -v rustup >/dev/null 2>&1; then \
	  echo "Installing rustup (https://rustup.rs) into $(CARGO_BIN) ..."; \
	  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain none; \
	fi
	@cd $(RUST) && (rustup toolchain install || rustup show active-toolchain)
	@cd $(RUST) && cargo --version && rustc --version
	@echo 'Rust is ready. Next: make build, then eval "$$(make -s env)" to put the programs on your PATH.'

build:
	cd $(RUST) && $(CAP) cargo build --release

test:
	cd $(RUST) && $(CAP) cargo test --release

env:
	@echo 'export PATH="$(CARGO_BIN):$(BIN):$$PATH"'
	@echo 'export COMMANDER_SIM_ROOT="$(ROOT)"'

shell:
	@echo "A shell with cargo, commander-sim, commander-practice and cardgen on PATH (exit to leave)."
	@PATH="$(BIN):$$PATH" $${SHELL:-/bin/bash}

sim: build
	$(CAP) $(BIN)/commander-sim --deck $(DECK) --pool $(POOL) --profile $(PROFILE) --games $(GAMES) --ai $(AI) \
	  --jobs $(JOBS) $(ARGS)

practice: build
	$(BIN)/commander-practice $(ARGS)

cardgen: build
	$(BIN)/cardgen $(ARGS)

clean:
	cd $(RUST) && cargo clean
