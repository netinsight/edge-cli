.POSIX:
.SUFFIXES:

NAME=edgectl
VERSION=$(shell git describe --always --match v[0-9]* HEAD)
VERSION_NUMBER=$(shell echo $(VERSION) | cut -c2-  )
OUT_DIR=build
PACKAGE_DIR=$(OUT_DIR)/$(NAME)-$(VERSION)

.PHONY: deb
deb: $(PACKAGE_DIR).deb

$(OUT_DIR):
	@mkdir -p "$@"

$(PACKAGE_DIR): \
	$(PACKAGE_DIR)/DEBIAN \
	$(PACKAGE_DIR)/usr/bin/$(NAME) \
	$(PACKAGE_DIR)/usr/share/bash-completion/completions/$(NAME) \
	$(PACKAGE_DIR)/usr/local/share/zsh/site-functions/$(NAME) \

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

$(PACKAGE_DIR)/usr/bin/$(NAME): edgectl/target/release/edgectl
	@mkdir -p "$(dir $@)"
	cp "$<" "$@" 

edgectl/target/release/edgectl: $(shell find edgectl/src) edgectl/Cargo.toml edgectl/Cargo.lock 
	cd edgectl && cargo build --release

$(PACKAGE_DIR)/usr/lib/$(NAME)/%: src/%
	@mkdir -p "$(dir $@)"
	cp -p "$<" "$@"

$(PACKAGE_DIR)/usr/share/bash-completion/completions/$(NAME): $(PACKAGE_DIR)/usr/bin/$(NAME)
	@mkdir -p "$(dir $@)"
	$< completion bash > $@

$(PACKAGE_DIR)/usr/local/share/zsh/site-functions/$(NAME): $(PACKAGE_DIR)/usr/bin/$(NAME)
	@mkdir -p "$(dir $@)"
	$< completion zsh > $@

$(PACKAGE_DIR).deb: $(PACKAGE_DIR)
	fakeroot dpkg-deb --build "${PACKAGE_DIR}"

.PHONY: clean
clean:
	rm -rf "$(OUT_DIR)"
