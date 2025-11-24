.POSIX:
.SUFFIXES:

NAME=edgectl
VERSION=$(shell git describe --always --match v[0-9]* HEAD)
VERSION_NUMBER=$(shell echo $(VERSION) | cut -c2-  )
OUT_DIR=build
PACKAGE_DIR=$(OUT_DIR)/$(NAME)-$(VERSION)

CARGO_ZIGBUILD_VERSION=0.19.7
CARGO_ZIGBUILD=docker run -e VERSION=$(VERSION) --rm -ti -v $(PWD):/io -w /io ghcr.io/rust-cross/cargo-zigbuild:${CARGO_ZIGBUILD_VERSION}

.PHONY: deb
deb: $(PACKAGE_DIR).deb

$(OUT_DIR):
	@mkdir -p "$@"

$(PACKAGE_DIR): \
	$(PACKAGE_DIR)/DEBIAN \
	$(PACKAGE_DIR)/usr/share/bash-completion/completions/$(NAME) \
	$(PACKAGE_DIR)/usr/local/share/zsh/site-functions/$(NAME) \
	$(PACKAGE_DIR)/usr/bin/$(NAME) \

	@touch "$@"

$(PACKAGE_DIR)/DEBIAN: \
	$(PACKAGE_DIR)/DEBIAN/conffile \
	$(PACKAGE_DIR)/DEBIAN/control \

	@touch "$@"

$(PACKAGE_DIR)/DEBIAN/control: debian/control
	(cat debian/control && echo -n 'Version: ' && echo "${VERSION_NUMBER}") > "$@"

$(PACKAGE_DIR)/DEBIAN/%: debian/%
	@mkdir -p "$(dir $@)"
	cp -p "debian/$*" "$@"

$(PACKAGE_DIR)/usr/share/bash-completion/completions/$(NAME): target/x86_64-unknown-linux-musl/release/$(NAME)
	@mkdir -p "$(dir $@)"
	COMPLETE=bash $< > $@

$(PACKAGE_DIR)/usr/local/share/zsh/site-functions/$(NAME): target/x86_64-unknown-linux-musl/release/$(NAME)
	@mkdir -p "$(dir $@)"
	COMPLETE=zsh $< > $@

$(PACKAGE_DIR)/usr/bin/$(NAME): target/x86_64-unknown-linux-musl/release/$(NAME)
	@mkdir -p "$(dir $@)"
	cp --link --force $< $@

$(PACKAGE_DIR).deb: $(PACKAGE_DIR)
	fakeroot dpkg-deb --build "${PACKAGE_DIR}"

target/x86_64-unknown-linux-musl/release/$(NAME): $(shell find src) Cargo.toml Cargo.lock
	$(CARGO_ZIGBUILD) cargo zigbuild --release --target x86_64-unknown-linux-musl
target/aarch64-apple-darwin/release/$(NAME): $(shell find src) Cargo.toml Cargo.lock
	$(CARGO_ZIGBUILD) cargo zigbuild --release --target aarch64-apple-darwin

$(OUT_DIR)/bin/$(NAME)-%: target/%/release/$(NAME) | $(OUT_DIR)/bin/
	cp $< $@

$(OUT_DIR)/ $(OUT_DIR)/bin/ $(OUT_DIR)/tmp/:
	mkdir -p $@

GH_VERSION=2.69.0
$(OUT_DIR)/tmp/gh: $(OUT_DIR)/tmp/gh_$(GH_VERSION)_linux_amd64/bin/gh
	cp --link --force $< $@
$(OUT_DIR)/tmp/gh_$(GH_VERSION)_linux_amd64.tar.gz: | $(OUT_DIR)/tmp/
	wget https://github.com/cli/cli/releases/download/v$(GH_VERSION)/gh_$(GH_VERSION)_linux_amd64.tar.gz -O $@
$(OUT_DIR)/tmp/gh_$(GH_VERSION)_linux_amd64/bin/gh: $(OUT_DIR)/tmp/gh_$(GH_VERSION)_linux_amd64.tar.gz
	tar -xf $(OUT_DIR)/tmp/gh_$(GH_VERSION)_linux_amd64.tar.gz  --directory $(OUT_DIR)/tmp/ gh_$(GH_VERSION)_linux_amd64/bin/gh
	touch $@

release: $(OUT_DIR)/tmp/gh \
	$(OUT_DIR)/bin/$(NAME)-x86_64-unknown-linux-musl \
	$(OUT_DIR)/bin/$(NAME)-aarch64-apple-darwin \
	$(PACKAGE_DIR).deb \

	$(OUT_DIR)/tmp/gh release create --verify-tag --notes-from-tag "$(VERSION)" \
		"$(PACKAGE_DIR).deb" \
		$(OUT_DIR)/bin/$(NAME)-x86_64-unknown-linux-musl \
		$(OUT_DIR)/bin/$(NAME)-aarch64-apple-darwin

.PHONY: clean
clean:
	rm -rf "$(OUT_DIR)" target
