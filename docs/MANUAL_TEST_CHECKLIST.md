# Manual test checklist

## Linux
- [ ] `sudo modprobe v4l2loopback devices=1 video_nr=10 card_label='BerryDesk Camera' exclusive_caps=1`
- [ ] `/dev/video10` exists and reports "BerryDesk Camera".
- [ ] `berryd` starts; `berry doctor` shows device_exists true.
- [ ] Add test pattern, start; Chrome/Firefox/OBS list "BerryDesk Camera" and show frames.
- [ ] Reconnect on feed drop; fallback image displays.
- [ ] Memory/CPU stable over soak.

## Windows
- [ ] Installer installs MF virtual camera; appears in Zoom/Teams/Meet/OBS/Chrome/Firefox.
- [ ] DirectShow fallback for older apps.

## macOS
- [ ] System extension approved in System Settings; camera appears in FaceTime/Safari/Chrome/OBS.
EOF
echo done