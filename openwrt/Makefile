# OpenWrt image for the Pi Zero W that fronts the chumby.
#
#   WAN = the "wwan" interface, IPv4 only, zone input ACCEPT. Ships with no
#         radio configured: join the wifi by hand from LuCI and assign it to
#         wwan. The build carries no wifi credentials.
#   LAN = usb0, USB-gadget ethernet (CDC-ECM) the chumby uses as its NIC,
#         and where LuCI is reachable at 192.168.7.1.
# Engineering record: ../claude/pi-as-chumby-nic.md (this route was
# abandoned - OpenWrt was unstable on this hardware; kept as a record)

VERSION   := 25.12.5
TARGET    := bcm27xx
SUBTARGET := bcm2708
PROFILE   := rpi

IB      := openwrt-imagebuilder-$(VERSION)-$(TARGET)-$(SUBTARGET).Linux-x86_64
IB_URL  := https://downloads.openwrt.org/releases/$(VERSION)/targets/$(TARGET)/$(SUBTARGET)/$(IB).tar.zst
IB_SHA  := 0d6b690907e9e25f46a6d811244f476bef212bdc97f015f2a7eb531c501b20ef

# usb_f_ecm is all the stock modules.d autoloads; the gadget itself (g_ether)
# is pulled in by files/etc/modules.d/60-g-ether.
# luci: the radio is configured on the device, so the box needs a UI for it.
PACKAGES := kmod-usb-dwc2 kmod-usb-gadget-eth luci

# The OTG port defaults to host mode, so the gadget needs the dwc2 driver in
# peripheral mode. Lives on the FAT partition, which FILES= cannot reach —
# patched into the Image Builder's config.txt template instead.
DWC2_LINE := dtoverlay=dwc2,dr_mode=peripheral

BUILD    := build
STAGE    := $(BUILD)/files
IMG      := $(BUILD)/$(IB)/bin/targets/$(TARGET)/$(SUBTARGET)/openwrt-$(VERSION)-$(TARGET)-$(SUBTARGET)-$(PROFILE)-squashfs-factory.img.gz
OUT      := /tmp/fw.bin

-include local.mk

.PHONY: all image stage clean distclean verify
all: $(OUT)

# No silent defaults: an unset credential must stop the build, not produce an
# image that quietly cannot reach the network.
define need
$(if $($(1)),,$(error $(1) is not set — copy local.mk.example to local.mk and fill it in))
endef

$(BUILD)/$(IB).tar.zst:
	mkdir -p $(BUILD)
	curl -fL -o $@ $(IB_URL)

$(BUILD)/$(IB)/.unpacked: $(BUILD)/$(IB).tar.zst
	echo "$(IB_SHA)  $<" | sha256sum -c -
	tar --zstd -xf $< -C $(BUILD)
	mkdir -p $(BUILD)/$(IB)/tmp
	touch $@

# Idempotent: re-running must not stack duplicate overlay lines.
$(BUILD)/$(IB)/.dwc2: $(BUILD)/$(IB)/.unpacked
	grep -qxF '$(DWC2_LINE)' $(BUILD)/$(IB)/target/linux/bcm27xx/image/config.txt \
	  || printf '\n# chumby: OTG port drives the LAN-side gadget NIC\n%s\n' '$(DWC2_LINE)' \
	     >> $(BUILD)/$(IB)/target/linux/bcm27xx/image/config.txt
	touch $@

stage:
	$(call need,ROOT_AUTHORIZED_KEYS)
	rm -rf $(STAGE)
	mkdir -p $(STAGE)
	cp -a files/. $(STAGE)/
	cat $(ROOT_AUTHORIZED_KEYS) > $(STAGE)/etc/dropbear/authorized_keys
	chmod 0600 $(STAGE)/etc/dropbear/authorized_keys
	test -s $(STAGE)/etc/dropbear/authorized_keys

image: stage $(BUILD)/$(IB)/.dwc2
	$(MAKE) -C $(BUILD)/$(IB) image PROFILE=$(PROFILE) \
	    PACKAGES="$(PACKAGES)" FILES="$(CURDIR)/$(STAGE)"

$(OUT): image
	gunzip -c $(IMG) > $@
	@echo "wrote $@ ($$(du -h $@ | cut -f1)) — dd this to the microSD card"

verify: $(OUT)
	./verify-image.sh $(BUILD)/$(IB) $(OUT)

clean:
	rm -rf $(STAGE)
	rm -f $(BUILD)/$(IB)/bin/targets/$(TARGET)/$(SUBTARGET)/*.img.gz

distclean:
	rm -rf $(BUILD)
