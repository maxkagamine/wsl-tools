TARGETS=\
	x86_64-unknown-linux-gnu \
	x86_64-pc-windows-gnu

ALL_BUILD=$(addprefix build-,$(TARGETS))
ALL_TEST=$(addprefix test-,$(TARGETS))

.PHONY: build test $(ALL_BUILD) $(ALL_TEST)

build: test $(ALL_BUILD)
test: $(ALL_TEST)

$(ALL_BUILD): build-%: test-%
	cargo build --release --target $*

$(ALL_TEST): test-%:
	cargo test --target $*
