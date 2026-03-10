cd ~/code/rust/compare_rs/folder-compare
set -euo pipefail

# ===== 可按需修改 =====
BIN_NAME="fc-ui-slint"                       # release 下可执行文件名
APP_NAME="FolderCompare"                     # 应用名
BUNDLE_ID="cn.wendx.foldercompare"
VERSION="0.1.12"
TARGET_DIR="target/aarch64-apple-darwin/release"
DIST_DIR="dist"
APP_PATH="$DIST_DIR/$APP_NAME.app"
DMG_NAME="$APP_NAME-macos-arm64-$VERSION.dmg"
DMG_PATH="$DIST_DIR/$DMG_NAME"
VOL_NAME="$APP_NAME Installer"
# ======================

# 如需先编译（已编译可注释）
cargo build -p "$BIN_NAME" --release --target aarch64-apple-darwin

test -f "$TARGET_DIR/$BIN_NAME"

mkdir -p "$DIST_DIR"

# 1) 组装 .app
rm -rf "$APP_PATH"
mkdir -p "$APP_PATH/Contents/MacOS" "$APP_PATH/Contents/Resources"
cp "$TARGET_DIR/$BIN_NAME" "$APP_PATH/Contents/MacOS/$BIN_NAME"
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
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# 可选图标
if [ -f "docs/assets/icon.icns" ]; then
  cp "docs/assets/icon.icns" "$APP_PATH/Contents/Resources/AppIcon.icns"
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
hdiutil create \
  -volname "$VOL_NAME" \
  -srcfolder "$STAGE_DIR" \
  -ov -format UDZO \
  "$DMG_PATH"

# 可选：给 dmg 做签名（你有 Developer ID 证书时启用）
# codesign --force --sign "Developer ID Application: YOUR NAME (TEAMID)" "$DMG_PATH"

# 4) 额外打 zip（可选）
ditto -c -k --sequesterRsrc --keepParent "$APP_PATH" "$DIST_DIR/$APP_NAME-macos-arm64.zip"

# 清理
rm -rf "$STAGE_DIR"

echo "DONE:"
echo "App: $APP_PATH"
echo "DMG: $DMG_PATH"
echo "ZIP: $DIST_DIR/$APP_NAME-macos-arm64.zip"
