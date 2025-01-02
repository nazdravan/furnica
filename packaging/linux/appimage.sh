#!/usr/bin/env bash
# AppImage pentru Linux x86_64: dist/release/furnica-<versiune>-linux-x86_64.AppImage
# Un singur fișier: se face executabil (chmod +x) și se pornește, fără instalare.
# appimagetool se descarcă în .cache/ la prima rulare.
set -euo pipefail
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

TOOL_URL=https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
tool=$ROOT/.cache/appimagetool-x86_64.AppImage
if [ ! -x "$tool" ]; then
    mkdir -p "$ROOT/.cache"
    curl -fsSL -o "$tool.part" "$TOOL_URL"
    chmod +x "$tool.part"
    mv "$tool.part" "$tool"
fi

cargo build --release --locked

name=$APP-$VERSION-linux-x86_64
appdir=$ROOT/dist/stage/Furnica.AppDir
rm -rf "$appdir" "$OUT/$name.AppImage"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" \
    "$appdir/usr/share/icons/hicolor/256x256/apps" "$appdir/usr/share/doc/furnica" "$OUT"
install -m 755 target/release/furnica "$appdir/usr/bin/furnica"
cp packaging/linux/furnica.desktop "$appdir/usr/share/applications/"
cp assets/icon.png "$appdir/usr/share/icons/hicolor/256x256/apps/furnica.png"
cp README.md LICENSE "$appdir/usr/share/doc/furnica/"
cp assets/fonts/LICENSE.txt "$appdir/usr/share/doc/furnica/LICENTE.txt"
# appimagetool cere fișierul .desktop și iconița și în rădăcina AppDir.
ln -s usr/share/applications/furnica.desktop "$appdir/furnica.desktop"
ln -s usr/share/icons/hicolor/256x256/apps/furnica.png "$appdir/furnica.png"
ln -s furnica.png "$appdir/.DirIcon"
ln -s usr/bin/furnica "$appdir/AppRun"

# Fără FUSE (containere, CI): appimagetool se dezarhivează și rulează din dosar.
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 VERSION=$VERSION \
    "$tool" --no-appstream "$appdir" "$OUT/$name.AppImage"
echo "Gata: $OUT/$name.AppImage"
