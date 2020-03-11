ifndef CONFIG
CONFIG := config.mk
endif

# -include config.mk
-include $(CONFIG)

BUILD_DIR := target/debug
BUILD_CMD := cargo build

H_FILES := "quantizr.h"

LINK_IMAGEQUANT_PC :=

ifneq ($(DEBUG),1)
BUILD_DIR := target/release
BUILD_CMD := RUSTFLAGS='-C link-arg=-s' $(BUILD_CMD) --release
endif

ifeq ($(IMAGEQUANT_COMPAT),1)
BUILD_CMD := $(BUILD_CMD) --features imagequant_compat
H_FILES := $(H_FILES) libimagequant.h
LINK_IMAGEQUANT_PC := ln -s quantizr.pc $(PKGCONFIGDIR)/imagequant.pc
endif

PKGCONFIG := quantizr.pc
CONTROL := control


all: build

build:
	$(BUILD_CMD)

install: $(PKGCONFIG)
	install -d $(LIBDIR)
	install -d $(PKGCONFIGDIR)
	install -d $(INCLUDEDIR)
	install -m 644 $(BUILD_DIR)/libquantizr.so $(LIBDIR)
	install -m 644 assets/$(PKGCONFIG) $(PKGCONFIGDIR)
	for f in $(H_FILES) ; do \
		install -m 644 assets/$$f $(INCLUDEDIR) ; \
	done
	$(LINK_IMAGEQUANT_PC)

build-deb:
	rm -rf dist
	mkdir dist

	#
	# Build standard version =====================================================
	#

	$(eval DEB_DIR := dist/quantizr_$(DEB_VERSION))

	CONFIG=dist/config.mk \
	./configure \
		--prefix=/usr \
		--libdir=$(DEB_DIR)/usr/lib \
		--pkgconfigdir=$(DEB_DIR)/usr/lib/pkgconfig \
		--includedir=$(DEB_DIR)/usr/include

	CONFIG=dist/config.mk make build $(CONTROL) install

	rm -rf dist/config.mk

	mkdir -p $(DEB_DIR)/DEBIAN
	cp assets/control $(DEB_DIR)/DEBIAN

	dpkg-deb --build $(DEB_DIR) $(DEB_DIR).deb

	#
	# Build imagequant-compat version ============================================
	#

	$(eval DEB_DIR := dist/quantizr-imagequant-compat_$(DEB_VERSION))

	CONFIG=dist/config.mk \
	./configure \
		--prefix=/usr \
		--libdir=$(DEB_DIR)/usr/lib \
		--pkgconfigdir=$(DEB_DIR)/usr/lib/pkgconfig \
		--includedir=$(DEB_DIR)/usr/include \
		--enable-imagequant-compatibility \
		--deb-name=quantizr-imagequant-compat

	CONFIG=dist/config.mk make build $(CONTROL) install

	rm -rf dist/config.mk

	mkdir -p $(DEB_DIR)/DEBIAN
	cp assets/control $(DEB_DIR)/DEBIAN

	dpkg-deb --build $(DEB_DIR) $(DEB_DIR).deb

build-deb-docker:
	docker run --rm -it -v $(shell pwd):/app -w /app rust:1-buster make build-deb

$(PKGCONFIG):
	cat assets/$(PKGCONFIG).in | sed 's|PREFIX|$(PREFIX)|;s|VERSION|$(VERSION)|' > assets/$(PKGCONFIG)

$(CONTROL):
	cat assets/$(CONTROL).in | sed 's|PREFIX|$(PREFIX)|;s|VERSION|$(DEB_VERSION)|;s|NAME|$(DEB_NAME)|' > assets/$(CONTROL)


