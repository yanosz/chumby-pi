#!/usr/bin/env bash
set -eu

DEV_ADDR=b8:27:eb:5c:f7:9d
HOST_ADDR=b8:27:eb:5c:f7:9e
LINK_NET=10.12.194.0/28

echo "options g_ether dev_addr=$DEV_ADDR host_addr=$HOST_ADDR" \
	> /etc/modprobe.d/usb-gadget-mac.conf
modprobe -r g_ether 2>/dev/null || true
modprobe g_ether
# https://github.com/raspberrypi/trixie-feedback/issues/62
cat > /etc/nftables.conf <<EOF
#!/usr/sbin/nft -f

flush ruleset

table ip chumby_nat {
	chain postrouting {
		type nat hook postrouting priority srcnat; policy accept;
		ip saddr $LINK_NET oifname != "usb0" masquerade
	}
}
EOF

echo 'net.ipv4.ip_forward = 1' > /etc/sysctl.d/99-chumby-forward.conf
sysctl -q -w net.ipv4.ip_forward=1
nft -f /etc/nftables.conf
systemctl enable nftables.service
