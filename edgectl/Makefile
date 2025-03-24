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

build/ build/bin/ build/tmp/:
	mkdir -p $@

GH_VERSION=2.69.0
build/tmp/gh: build/tmp/gh_$(GH_VERSION)_linux_amd64/bin/gh
	cp --link $< $@
build/tmp/gh_$(GH_VERSION)_linux_amd64.tar.gz: | build/tmp/
	wget https://github.com/cli/cli/releases/download/v$(GH_VERSION)/gh_$(GH_VERSION)_linux_amd64.tar.gz -O $@
build/tmp/gh_$(GH_VERSION)_linux_amd64/bin/gh: build/tmp/gh_$(GH_VERSION)_linux_amd64.tar.gz
	tar -xf build/tmp/gh_$(GH_VERSION)_linux_amd64.tar.gz  --directory build/tmp/ gh_$(GH_VERSION)_linux_amd64/bin/gh

release: build/tmp/gh build-all \
	build/bin/edgectl-x86_64-unknown-linux-musl \
	build/bin/edgectl-aarch64-apple-darwin
	build/tmp/gh release create --notes-from-tag $(shell git tag --points-at HEAD)
	build/tmp/gh release upload $(shell git tag --points-at HEAD) build/bin/edgectl-x86_64-unknown-linux-musl build/bin/edgectl-aarch64-apple-darwin

.PHONY: clean
clean:
	rm -rf target/ build/
