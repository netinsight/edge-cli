.SUFFIXES:

.PHONY: build-all
build-all: \
	build/bin/edgectl-x86_64-unknown-linux-musl \
	build/bin/edgectl-aarch64-apple-darwin

CARGO_ZIGBUILD_VERSION=0.19.7
CARGO_ZIGBUILD=docker run --rm -ti -v $(PWD):/io -w /io ghcr.io/rust-cross/cargo-zigbuild:${CARGO_ZIGBUILD_VERSION}

.PHONY: target/x86_64-unknown-linux-musl/release/edgectl
.PHONY: target/aarch64-apple-darwin/release/edgectl
target/x86_64-unknown-linux-musl/release/edgectl:
	$(CARGO_ZIGBUILD) cargo zigbuild --release --target x86_64-unknown-linux-musl
target/aarch64-apple-darwin/release/edgectl:
	$(CARGO_ZIGBUILD) cargo zigbuild --release --target aarch64-apple-darwin

build/bin/edgectl-%: target/%/release/edgectl | build/bin/
	cp $< $@

build/ build/bin/:
	mkdir -p $@

.PHONY: clean
clean:
	rm -rf target/ build/
