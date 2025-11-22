#!/bin/bash

set -e

echo "Updating earctl AUR package..."
echo ""

# Generate .SRCINFO
echo "Generating .SRCINFO..."
makepkg --printsrcinfo > .SRCINFO
echo ""

# Copy to AUR repo
echo "Copying files to AUR repository..."
AUR_DIR="../earctl-aur"
cp PKGBUILD "$AUR_DIR/"
cp .SRCINFO "$AUR_DIR/"
cp earctl.install "$AUR_DIR/"
echo ""

# Show diff
echo "Changes to be committed:"
cd "$AUR_DIR"
git diff
echo ""

echo "Ready to push!"
echo ""
echo "cd $AUR_DIR"
echo "git add PKGBUILD .SRCINFO"
echo "git commit -m 'Add systemd service and automatic startup (pkgrel=2)'"
echo "git push"
