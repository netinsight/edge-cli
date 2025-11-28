.POSIX:
.SUFFIXES:

NAME=edgectl
VERSION=$(shell git describe --always --match v[0-9]* HEAD)
VERSION_NUMBER=$(shell echo $(VERSION) | cut -c2-  )
OUT_DIR=build
PACKAGE_DIR=$(OUT_DIR)/$(NAME)-$(VERSION)
SECRET_KEY_PATH ?= ~/Documents/certificates

ifeq ($(shell uname -sm),Darwin arm64)
  CARGO_NATIVE_TARGET=aarch64-apple-darwin
  GH_ARCH=macOS_arm64
  GH_EXT=zip
else
  CARGO_NATIVE_TARGET=x86_64-unknown-linux-musl
  GH_ARCH=linux_amd64
  GH_EXT=tar.gz
endif

CARGO_ZIGBUILD_VERSION=0.19.7
CARGO_ZIGBUILD=docker run -e VERSION=$(VERSION) --rm -ti -v $(PWD):/io -w /io ghcr.io/rust-cross/cargo-zigbuild:$(CARGO_ZIGBUILD_VERSION)

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
	(cat debian/control && echo "Version: $(VERSION_NUMBER)") > "$@"

$(PACKAGE_DIR)/DEBIAN/%: debian/%
	@mkdir -p "$(dir $@)"
	cp -p "debian/$*" "$@"

$(PACKAGE_DIR)/usr/share/bash-completion/completions/$(NAME): target/$(CARGO_NATIVE_TARGET)/release/$(NAME)
	@mkdir -p "$(dir $@)"
	COMPLETE=bash $< > $@

$(PACKAGE_DIR)/usr/local/share/zsh/site-functions/$(NAME): target/$(CARGO_NATIVE_TARGET)/release/$(NAME)
	@mkdir -p "$(dir $@)"
	COMPLETE=zsh $< > $@

$(PACKAGE_DIR)/usr/bin/$(NAME): target/x86_64-unknown-linux-musl/release/$(NAME)
	@mkdir -p "$(dir $@)"
	cp -l -f $< $@

$(PACKAGE_DIR).deb: $(PACKAGE_DIR)
	fakeroot dpkg-deb --build "$(PACKAGE_DIR)"

$(OUT_DIR)/bin/rcodesign:
	cargo install apple-codesign --root $(OUT_DIR) --version 0.29.0

target/x86_64-unknown-linux-musl/release/$(NAME): $(shell find src) Cargo.toml Cargo.lock
	$(CARGO_ZIGBUILD) cargo zigbuild --release --target x86_64-unknown-linux-musl
target/aarch64-apple-darwin/release/$(NAME): $(shell find src) Cargo.toml Cargo.lock $(OUT_DIR)/bin/rcodesign \
		$(SECRET_KEY_PATH)/cert.p12 $(SECRET_KEY_PATH)/cert-secret.txt $(SECRET_KEY_PATH)/api-key.json
	$(CARGO_ZIGBUILD) cargo zigbuild --release --target aarch64-apple-darwin
	$(OUT_DIR)/bin/rcodesign sign \
		--p12-file $(SECRET_KEY_PATH)/cert.p12 \
		--p12-password-file $(SECRET_KEY_PATH)/cert-secret.txt \
		--for-notarization \
		$@
	zip -j $(OUT_DIR)/bin/$(NAME)-aarch64-apple-darwin.zip $@
	$(OUT_DIR)/bin/rcodesign notary-submit \
		--api-key-file $(SECRET_KEY_PATH)/api-key.json \
		--wait \
		$(OUT_DIR)/bin/$(NAME)-aarch64-apple-darwin.zip
	rm $(OUT_DIR)/bin/$(NAME)-aarch64-apple-darwin.zip

$(OUT_DIR)/bin/$(NAME)-%: target/%/release/$(NAME) | $(OUT_DIR)/bin/
	cp $< $@

$(OUT_DIR)/ $(OUT_DIR)/bin/ $(OUT_DIR)/tmp/:
	mkdir -p $@

GH_VERSION=2.83.1
GH_ARCHIVE=$(OUT_DIR)/tmp/gh_$(GH_VERSION)_$(GH_ARCH).$(GH_EXT)
$(OUT_DIR)/tmp/gh: $(OUT_DIR)/tmp/gh_$(GH_VERSION)_$(GH_ARCH)/bin/gh
	cp -f $< $@
$(GH_ARCHIVE): | $(OUT_DIR)/tmp/
	curl -L https://github.com/cli/cli/releases/download/v$(GH_VERSION)/gh_$(GH_VERSION)_$(GH_ARCH).$(GH_EXT) -o $@
ifeq ($(GH_EXT),zip)
$(OUT_DIR)/tmp/gh_$(GH_VERSION)_$(GH_ARCH)/bin/gh: $(GH_ARCHIVE)
	unzip -o $(GH_ARCHIVE) gh_$(GH_VERSION)_$(GH_ARCH)/bin/gh -d $(OUT_DIR)/tmp/
	touch $@
else
$(OUT_DIR)/tmp/gh_$(GH_VERSION)_$(GH_ARCH)/bin/gh: $(GH_ARCHIVE)
	tar -xf $(GH_ARCHIVE) --directory $(OUT_DIR)/tmp/ gh_$(GH_VERSION)_$(GH_ARCH)/bin/gh
	touch $@
endif

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
