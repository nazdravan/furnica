#!/usr/bin/env bash
# Pachetul .deb pentru Ubuntu / Debian / Mint: dist/release/furnica-<versiune>-linux-x86_64.deb
# Se instalează cu `sudo apt install ./furnica-*.deb`; apare în meniu, iar
# fișierele .ant se deschid cu dublu-click. Cere dpkg-deb (pachetul dpkg).
set -euo pipefail
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

cargo build --release --locked

name=$APP-$VERSION-linux-x86_64
root=$ROOT/dist/stage/deb
rm -rf "$root" "$OUT/$name.deb"
mkdir -p "$root/DEBIAN" "$root/usr/bin" "$root/usr/share/applications" \
    "$root/usr/share/icons/hicolor/256x256/apps" "$root/usr/share/mime/packages" \
    "$root/usr/share/doc/furnica/exemple" "$OUT"
install -m 755 target/release/furnica "$root/usr/bin/furnica"
install -m 644 packaging/linux/furnica.desktop "$root/usr/share/applications/furnica.desktop"
install -m 644 assets/icon.png "$root/usr/share/icons/hicolor/256x256/apps/furnica.png"
install -m 644 packaging/linux/furnica-mime.xml "$root/usr/share/mime/packages/furnica.xml"
install -m 644 examples/*.ant "$root/usr/share/doc/furnica/exemple/"
install -m 644 README.md "$root/usr/share/doc/furnica/README.md"
{
    echo "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/"
    echo "Upstream-Name: furnica"
    echo "Source: https://github.com/nazdravan/furnica"
    echo
    echo "Files: *"
    echo "License: GPL-3.0-or-later"
    echo " On Debian systems, the full text is in /usr/share/common-licenses/GPL-3."
    echo
    echo "Files: fonts"
    echo "License: OFL-1.1"
    sed 's/^/ /; s/^ $/ ./' assets/fonts/LICENSE.txt
} >"$root/usr/share/doc/furnica/copyright"

# Versiunile Debian folosesc ~ pentru pre-release-uri: 3.1.0-rc.1 -> 3.1.0~rc.1.
debver=${VERSION/-/\~}
size=$(du -sk "$root/usr" | cut -f1)
cat >"$root/DEBIAN/control" <<CONTROL
Package: furnica
Version: $debver
Architecture: amd64
Maintainer: nazdravan <nazdravan@users.noreply.github.com>
Installed-Size: $size
Depends: libc6 (>= 2.35), libgl1, libxkbcommon0
Recommends: libxkbcommon-x11-0, libwayland-client0
Section: education
Priority: optional
Homepage: https://github.com/nazdravan/furnica
Description: executant pentru învățarea algoritmilor
 Elevul scrie un program cu comenzi simple, iar furnica îl execută pe o
 tabelă cu litere, împingând literele din calea ei. Rescris în Rust după
 programul original „Furnica” (Ant v2.00, 1999); deschide fișierele .ant
 vechi. Interfața în română, rusă și engleză.
CONTROL

dpkg-deb --root-owner-group -Zxz --build "$root" "$OUT/$name.deb"
echo "Gata: $OUT/$name.deb"
