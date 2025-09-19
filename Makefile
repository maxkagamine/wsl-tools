INNO?=/mnt/c/Program\ Files\ \(x86\)/Inno\ Setup\ 6/ISCC.exe

BIN_NAMES:=$(basename $(notdir $(wildcard src/bin/*)))
WINDOWS_ONLY_BINS:=code-wsl run-in-wsl

TARGET_LINUX:=x86_64-unknown-linux-gnu
TARGET_WINDOWS:=x86_64-pc-windows-gnu

BINS_LINUX:=$(addprefix target/$(TARGET_LINUX)/release/,$(filter-out $(WINDOWS_ONLY_BINS),$(BIN_NAMES)))
BINS_WINDOWS:=$(addprefix target/$(TARGET_WINDOWS)/release/,$(addsuffix .exe,$(BIN_NAMES)))
ALL_BINS:=$(BINS_LINUX) $(BINS_WINDOWS)

TESTS_LINUX:=target/$(TARGET_LINUX)/.test
TESTS_WINDOWS:=target/$(TARGET_WINDOWS)/.test
ALL_TESTS:=$(TESTS_LINUX) $(TESTS_WINDOWS)

SOURCE_FILES:=$(shell fd '\.rs$$') Cargo.toml Cargo.lock

# Installer & zip file
dist/wsl-tools-installer.exe: $(ALL_BINS) LICENSE.txt Setup.iss
	rm -rf dist && mkdir -p dist/wsl-tools
	cp $(ALL_BINS) LICENSE.txt dist/wsl-tools
ifeq ($(wildcard $(INNO)),)
	$(warning ⚠️  Inno Setup 6 not installed, skipping installer ⚠️ )
else
	$(INNO) Setup.iss
endif
	cd dist && rm -f wsl-tools-portable.zip && zip -r wsl-tools-portable.zip wsl-tools
	rm -rf dist/wsl-tools

# Builds for each bin name & target depending on tests, which in turn depend on **/*.rs
target/$(TARGET_LINUX)/release/%: $(TESTS_LINUX)
	cargo build --release --target $(TARGET_LINUX) --bin $*

target/$(TARGET_WINDOWS)/release/%.exe: $(TESTS_WINDOWS)
	cargo build --release --target $(TARGET_WINDOWS) --bin $*

# Tests for each target depending on **/*.rs, using an empty file to mark the last successful run
target/%/.test: $(SOURCE_FILES)
	cargo clippy --target $* -- -Wclippy::pedantic
	cargo test --target $* -- --test-threads=1
	@touch target/$*/.test

.PHONY: test
test: $(ALL_TESTS)
