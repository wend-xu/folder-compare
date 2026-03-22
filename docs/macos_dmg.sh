#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# ===== 可按需修改 =====
BIN_NAME="fc-ui-slint"                       # release 下可执行文件名
APP_NAME="FolderCompare"                     # 应用名
BUNDLE_ID="cn.wendx.foldercompare"
MIN_SYSTEM_VERSION="12.0"
TARGET_TRIPLE="${TARGET_TRIPLE:-aarch64-apple-darwin}"
ARCHIVE_SUFFIX="macos-arm64"
VERSION="$(cargo pkgid -p "$BIN_NAME" | sed 's/.*[#@]//')"
TARGET_DIR="$ROOT_DIR/target/$TARGET_TRIPLE/release"
DIST_DIR="$ROOT_DIR/dist"
APP_PATH="$DIST_DIR/$APP_NAME.app"
DMG_NAME="$APP_NAME-$ARCHIVE_SUFFIX-$VERSION.dmg"
DMG_PATH="$DIST_DIR/$DMG_NAME"
ZIP_NAME="$APP_NAME-$ARCHIVE_SUFFIX-$VERSION.zip"
ZIP_PATH="$DIST_DIR/$ZIP_NAME"
VOL_NAME="$APP_NAME Installer"
ICON_PATH="$ROOT_DIR/docs/assets/icon.icns"
BIN_PATH="$TARGET_DIR/$BIN_NAME"
# ======================

STAGE_DIR=""
cleanup() {
  if [ -n "${STAGE_DIR:-}" ] && [ -d "$STAGE_DIR" ]; then
    rm -rf "$STAGE_DIR"
  fi
}
trap cleanup EXIT

# 如需先编译（已编译可注释）
cargo build -p "$BIN_NAME" --release --target "$TARGET_TRIPLE"

test -f "$BIN_PATH"

mkdir -p "$DIST_DIR"

# 1) 组装 .app
rm -rf "$APP_PATH"
mkdir -p "$APP_PATH/Contents/MacOS" "$APP_PATH/Contents/Resources"
cp "$BIN_PATH" "$APP_PATH/Contents/MacOS/$BIN_NAME"
chmod +x "$APP_PATH/Contents/MacOS/$BIN_NAME"

cat > "$APP_PATH/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$APP_NAME</string>
  <key>CFBundleDisplayName</key><string>$APP_NAME</string>
  <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleExecutable</key><string>$BIN_NAME</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>$MIN_SYSTEM_VERSION</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# 可选图标
if [ -f "$ICON_PATH" ]; then
  cp "$ICON_PATH" "$APP_PATH/Contents/Resources/AppIcon.icns"
  /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string AppIcon.icns" "$APP_PATH/Contents/Info.plist" \
    || /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile AppIcon.icns" "$APP_PATH/Contents/Info.plist"
fi

# ad-hoc 签名（本机可运行）
codesign --force --deep --sign - "$APP_PATH"
xattr -cr "$APP_PATH"

# 2) 准备 DMG staging
STAGE_DIR="$(mktemp -d /tmp/${APP_NAME}.dmg.stage.XXXXXX)"
cp -R "$APP_PATH" "$STAGE_DIR/"
ln -s /Applications "$STAGE_DIR/Applications"

# 3) 生成可分发 DMG（UDZO 压缩）
rm -f "$DMG_PATH"
test -d "$STAGE_DIR"
test -d "$DIST_DIR"
hdiutil create \
  -volname "$VOL_NAME" \
  -srcfolder "$STAGE_DIR" \
  -ov -format UDZO \
  "$DMG_PATH"

# 可选：给 dmg 做签名（你有 Developer ID 证书时启用）
# codesign --force --sign "Developer ID Application: YOUR NAME (TEAMID)" "$DMG_PATH"

# 4) 额外打 zip（可选）
rm -f "$ZIP_PATH"
ditto -c -k --sequesterRsrc --keepParent "$APP_PATH" "$ZIP_PATH"

echo "DONE:"
echo "App: $APP_PATH"
echo "DMG: $DMG_PATH"
echo "ZIP: $ZIP_PATH"
