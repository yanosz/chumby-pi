#!/bin/sh
# Check the built image for the handful of things that decide whether the box
# comes back on the network. Reads the image itself, not the staging tree, so
# what is asserted is what gets dd'd.
#
# usage: verify-image.sh <imagebuilder-dir> <raw-image>
set -u

IB="$1"
IMG="$2"
HOST="$IB/staging_dir/host/bin"
BOOT_OFFSET=4194304   # 8192 sectors, per gen_rpi_sdcard_img.sh
fail=0

check() {
	if [ "$1" = 0 ]; then
		echo "  ok   $2"
	else
		echo "  FAIL $2"
		fail=1
	fi
}

fatal() { echo "  ABORT $1"; exit 2; }

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "boot partition:"
"$HOST/mcopy" -n -i "$IMG@@$BOOT_OFFSET" ::config.txt "$work/config.txt" ||
	fatal "cannot read the boot partition"
grep -q '^dtoverlay=dwc2,dr_mode=peripheral$' "$work/config.txt"
check $? "config.txt selects dwc2 in peripheral mode"

echo "root filesystem:"
# Partition 2 starts at 147456 sectors; squashfs begins there.
dd if="$IMG" of="$work/root.squashfs" bs=512 skip=147456 status=none
# Only the paths asserted below: a full extract needs root for /dev nodes.
"$HOST/unsquashfs4" -q -d "$work/root" "$work/root.squashfs" \
	etc lib/modules www >/dev/null 2>&1 || fatal "cannot unpack the rootfs"
R="$work/root"

grep -q "^g_ether$" "$R/etc/modules.d/60-g-ether"
check $? "g_ether autoloads (stock package only autoloads usb_f_ecm)"

test -f "$R/lib/modules/"*"/g_ether.ko" && test -f "$R/lib/modules/"*"/dwc2.ko"
check $? "g_ether.ko and dwc2.ko present"

test -s "$R/etc/config/network" && test -s "$R/etc/config/system"
check $? "network+system shipped, so config_generate leaves them alone"

grep -q "option device 'br-lan'" "$R/etc/config/network" &&
	grep -q "option bridge_empty '1'" "$R/etc/config/network" &&
	grep -q "list ports 'usb0'" "$R/etc/config/network"
check $? "lan is br-lan, keeps its address with no port"

test -x "$R/etc/rc.local" && grep -q "/dev/console" "$R/etc/rc.local"
check $? "boot-time state dump to console"

awk "/^config dhcp 'lan'/{l=1;next} /^config /{l=0} l&&/option start/{f=1} END{exit !f}" \
	"$R/etc/config/dhcp"
check $? "dhcp pool configured on the gadget link"

test -z "$(sed -n '/config interface .wwan6./p' "$R/etc/config/network")"
check $? "no wwan6 — wan stays IPv4-only"

awk '/^config zone/{z=""} /option name .wan./{z="wan"} z=="wan" && /option input .ACCEPT./{found=1} END{exit !found}' \
	"$R/etc/config/firewall"
check $? "wan zone accepts all input"

grep -q "option PasswordAuth 'off'" "$R/etc/config/dropbear" &&
	grep -q "option RootPasswordAuth 'off'" "$R/etc/config/dropbear"
check $? "dropbear is key-only"

test -s "$R/etc/dropbear/authorized_keys"
check $? "authorized_keys is not empty"

[ "$(stat -c %a "$R/etc/dropbear/authorized_keys")" = 600 ]
check $? "authorized_keys is 0600"

# The radio is configured by hand on the device. Nothing in the image may
# carry wifi settings, so assert their absence rather than their contents.
test ! -e "$R/etc/config/wireless"
check $? "no wireless config shipped"

test -z "$(grep -rlE "encryption|wireless\." "$R/etc/config" "$R/etc/uci-defaults" 2>/dev/null)"
check $? "no wifi settings anywhere in etc"

# LuCI has been JS-on-rpcd since 21.02, so there is no /usr/lib/lua to look
# for — the markers are the server, the ubus bridge and the static tree.
test -f "$R/etc/init.d/uhttpd" && test -f "$R/etc/init.d/rpcd" &&
	test -d "$R/www/luci-static" && test -f "$R/www/cgi-bin/luci"
check $? "luci is installed and served"

# The admin UI must not be listening on the wifi side.
grep -q "list listen_http '192.168.7.1:80'" "$R/etc/config/uhttpd" &&
	! grep -qE "listen_(http|https).*(0\.0\.0\.0|\[::\])" "$R/etc/config/uhttpd"
check $? "uhttpd listens on the lan address only"

# Removing the credentials must not have removed the ability to join a
# network: driver, firmware, nvram and supplicant all have to stay.
test -f "$R/lib/modules/"*"/brcmfmac.ko" && test -f "$R/etc/init.d/wpad"
check $? "brcmfmac driver and wpad present"

echo "packages:"
for p in kmod-usb-dwc2 kmod-usb-gadget-eth kmod-brcmfmac cypress-firmware-43430-sdio \
	brcmfmac-nvram-43430-sdio wpad-basic-mbedtls luci luci-mod-network uhttpd; do
	grep -q "^$p " "$IB/bin/targets/bcm27xx/bcm2708/"*.manifest
	check $? "$p installed"
done

exit $fail
