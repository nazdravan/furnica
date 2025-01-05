#!/usr/bin/env bash
# Aplicația pentru macOS: dist/release/furnica-<versiune>-macos-universal.zip
# Furnica.app universal (Apple Silicon + Intel). Se rulează pe un Mac (sau pe
# runner-ul macOS din GitHub Actions): are nevoie de Xcode Command Line Tools.
set -euo pipefail
. "$(dirname "$0")/../env.sh"
cd "$ROOT"

# Cel mai vechi macOS suportat (Rust cere cel puțin 10.12 pe Intel și 11 pe Apple Silicon).
export MACOSX_DEPLOYMENT_TARGET=10.13
targets=(aarch64-apple-darwin x86_64-apple-darwin)
if command -v rustup >/dev/null; then
    rustup target add "${targets[@]}"
fi
for t in "${targets[@]}"; do
    cargo build --release --locked --target "$t"
done

name=$APP-$VERSION-macos-universal
stage=$ROOT/dist/stage/$name
app=$stage/Furnica.app
rm -rf "$stage" "$OUT/$name.zip"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$stage/Exemple" "$OUT"
lipo -create -output "$app/Contents/MacOS/furnica" \
    target/aarch64-apple-darwin/release/furnica target/x86_64-apple-darwin/release/furnica
sed -e "s/@VERSION@/$VERSION/g" -e "s/@MIN_MACOS@/$MACOSX_DEPLOYMENT_TARGET/g" \
    packaging/macos/Info.plist >"$app/Contents/Info.plist"
cp assets/furnica.icns "$app/Contents/Resources/furnica.icns"
cp examples/*.ant "$stage/Exemple/"
cp README.md LICENSE packaging/macos/CITESTE.txt "$stage/"
cp assets/fonts/LICENSE.txt "$stage/LICENTE.txt"

# Semnătură ad-hoc: fără ea, pe Apple Silicon aplicația nici nu pornește.
# Nu înlocuiește semnarea cu un certificat Apple Developer (vezi CITESTE.txt).
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"
lipo -info "$app/Contents/MacOS/furnica"

# ditto păstrează atributele și semnătura pachetului .app.
ditto -c -k --keepParent "$stage" "$OUT/$name.zip"
echo "Gata: $OUT/$name.zip"
