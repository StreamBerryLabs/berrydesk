#!/usr/bin/env bash
set -euo pipefail
# Persistent v4l2loopback setup for BerryDesk.

# 1. Load now
modprobe v4l2loopback devices=1 video_nr=10 card_label='BerryDesk Camera' exclusive_caps=1

# 2. Auto-load on boot
echo 'v4l2loopback' > /etc/modules-load.d/v4l2loopback.conf

# 3. Module options
cat > /etc/modprobe.d/v4l2loopback.conf <<'HEREDOC'
options v4l2loopback devices=1 video_nr=10 card_label="BerryDesk Camera" exclusive_caps=1
HEREDOC

echo "Done. /dev/video10 is now 'BerryDesk Camera' and will load on every boot."
ls -l /dev/video10