# Fake feed rig

- `mjpeg_server.py`: HTTP MJPEG + snapshot server with optional auth, delay, frame drops.
  Requires Python 3 with numpy and Pillow.

Run:
  python3 mjpeg_server.py --port 8080
  berry add --name fake http://localhost:8080/video.mjpg
  berry start fake

For RTSP, run a local RTSP server (e.g. mediamtx) and publish a test pattern via ffmpeg.
