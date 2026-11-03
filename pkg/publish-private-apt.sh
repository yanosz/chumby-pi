#!/bin/bash
# Publish the private debs (chumby-rust-remote, chumby-ikea-lys) as a signed
# flat apt repo on the LAN's web server (claude/issues.md #31). The recipe is
# the public repo's (.github/workflows/ci.yml, "Build and sign the flat repo").
#
# Everything about the server lives outside this public repo, in
# ~/.config/chumby-apt/private.env: HOST, USER (ssh/rsync login), DIR (the
# repo directory on the server), URL (where apt reaches it), KEY_ID (the
# private signing key in gpg). Optional there: RUST_REMOTE_OUT, LYS_OUT —
# where the debs are built (defaults below). The public key the Pi trusts
# sits beside it as chumby-private-archive.gpg.
#
# Usage: pkg/publish-private-apt.sh [--dry-run]
#   --dry-run    build, sign and verify the index locally, list it, stop
#   --sources    print the Pi's setup (keyring + sources line) and stop
set -euo pipefail

CONF_DIR="$HOME/.config/chumby-apt"
CONF="$CONF_DIR/private.env"
PUBKEY="$CONF_DIR/chumby-private-archive.gpg"
REPO=$(cd "$(dirname "$0")/.." && pwd)

die() { echo "publish-private-apt: $*" >&2; exit 1; }

[ -r "$CONF" ] || die "no $CONF (HOST, USER, DIR, URL, KEY_ID)"
[ -r "$PUBKEY" ] || die "no $PUBKEY (the repo's public key)"
# Its own names: USER would otherwise be the login's.
CONF_VARS=$(set -a; . "$CONF"; for v in HOST USER DIR URL KEY_ID RUST_REMOTE_OUT LYS_OUT; do
    printf 'P_%s=%q\n' "$v" "$(eval "printf '%s' \"\${$v:-}\"")"; done)
eval "$CONF_VARS"
for v in HOST USER DIR URL KEY_ID; do
    eval "[ -n \"\$P_$v\" ]" || die "$v missing in $CONF"
done
case "$P_DIR" in
    /*/*) ;;
    *) die "DIR must be an absolute path below a directory, not '/' or a top-level one" ;;
esac
RUST_REMOTE_OUT="${P_RUST_REMOTE_OUT:-$REPO/rust-remote/pkg/out}"
LYS_OUT="${P_LYS_OUT:-/home/jan/chumby-ikea-lys/pkg/out}"

if [ "${1:-}" = "--sources" ]; then
    cat <<EOF
# On the Pi, once (as root):
install -m 644 chumby-private-archive.gpg /usr/share/keyrings/chumby-private-archive.gpg
#   (the file: $P_URL/chumby-private-archive.gpg)
echo 'deb [signed-by=/usr/share/keyrings/chumby-private-archive.gpg] $P_URL ./' \\
    > /etc/apt/sources.list.d/chumby-private.list
apt update && apt install chumby-rust-remote chumby-ikea-lys
EOF
    exit 0
fi

gpg --batch --list-secret-keys "$P_KEY_ID" >/dev/null 2>&1 || die "the signing key (KEY_ID) is not in gpg"

newest() {  # the newest deb of package $1 in directory $2, by version
    local pkg=$1 dir=$2 best="" bestv="" f v
    for f in "$dir/${pkg}"_*_arm64.deb; do
        [ -e "$f" ] || continue
        v=$(dpkg-deb -f "$f" Version)
        if [ -z "$best" ] || dpkg --compare-versions "$v" gt "$bestv"; then best=$f; bestv=$v; fi
    done
    [ -n "$best" ] || die "no ${pkg}_*_arm64.deb in $dir — build it (pkg/build-deb.sh there)"
    printf '%s\n' "$best"
}

STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
cp "$(newest chumby-rust-remote "$RUST_REMOTE_OUT")" "$(newest chumby-ikea-lys "$LYS_OUT")" "$STAGE/"
cp "$PUBKEY" "$STAGE/chumby-private-archive.gpg"
(
    cd "$STAGE"
    apt-ftparchive packages . > Packages
    gzip -9k Packages
    apt-ftparchive \
        -o APT::FTPArchive::Release::Origin=chumby-private \
        -o APT::FTPArchive::Release::Label=chumby-private \
        -o APT::FTPArchive::Release::Suite=stable \
        -o APT::FTPArchive::Release::Codename=stable \
        -o APT::FTPArchive::Release::Architectures=arm64 \
        -o "APT::FTPArchive::Release::Description=chumby-pi private packages" \
        release . > Release
    gpg --batch --yes --local-user "$P_KEY_ID" --clearsign -o InRelease Release
    gpg --batch --yes --local-user "$P_KEY_ID" --detach-sign --armor -o Release.gpg Release
    gpgv --keyring "$PUBKEY" InRelease
    gpgv --keyring "$PUBKEY" Release.gpg Release
)
echo "index built and verified:"
(cd "$STAGE" && ls -l | sed 1d && sha256sum ./*.deb)

[ "${1:-}" = "--dry-run" ] && exit 0

# The repo directory only: --delete never reaches above it.
chmod 755 "$STAGE"
rsync -rlpt --delete --chmod=D755,F644 -e "ssh -o BatchMode=yes" "$STAGE/" "$P_USER@$P_HOST:$P_DIR/"
echo "uploaded; fetch check:"
curl -fsS "$P_URL/InRelease" -o "$STAGE/fetched-InRelease" || die "InRelease not served"
gpgv --keyring "$PUBKEY" "$STAGE/fetched-InRelease"
curl -fsS "$P_URL/Packages" | grep -E '^(Package|Version):'
for f in Packages.gz Release Release.gpg chumby-private-archive.gpg $(cd "$STAGE" && ls ./*.deb); do
    f=${f#./}
    curl -fsS -o /dev/null -w "%{http_code} $f\n" "$P_URL/$f" || die "$f not served"
done
