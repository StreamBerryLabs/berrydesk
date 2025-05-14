# Windows virtual camera (Media Foundation + DirectShow)

Not built in this environment (Linux only). Implementation plan:

- Media Foundation virtual camera using MFCreateVirtualCamera (Windows 11).
- DirectShow filter fallback for Windows 10 / older apps.
- Signed COM media source DLL reading frames from the shared-memory frame bus.
- Installer: MSI/MSIX + winget.

Build requires Windows SDK, Visual Studio, code-signing certificate.
Manual test checklist: docs/MANUAL_TEST_CHECKLIST.md
EOF
cat > /home/swadhin/brrrydesk/berrydesk/platform/macos-vcam/README.md <<'EOF'
# macOS virtual camera (CoreMediaIO Camera Extension)

Not built in this environment. Implementation plan:

- Swift Camera Extension (system extension) using CoreMediaIO.
- Reads frames from shared-memory frame bus via XPC/IOSurface.
- Entitlements, notarization, user approval in System Settings.
- DMG + Homebrew.

Build requires Xcode, Apple Developer account, signing/notarization.
EOF
echo done