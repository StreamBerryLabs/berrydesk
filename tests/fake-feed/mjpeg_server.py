#!/usr/bin/env python3
"""Fake MJPEG feed server for testing BerryDesk without real cameras.

Serves a generated test pattern as multipart/x-mixed-replace MJPEG on /video.mjpg.
Supports optional basic auth, delay, and frame drops via query params or env vars.
"""
import argparse, time, os, io
from http.server import BaseHTTPRequestHandler, HTTPServer
import numpy as np

def make_frame(w, h, t):
    # simple animated test pattern (gradient + moving bar)
    img = np.zeros((h, w, 3), dtype=np.uint8)
    for y in range(h):
        img[y, :, 0] = (y + int(t*30)) % 256
    for x in range(w):
        img[:, x, 1] = (x + int(t*20)) % 256
    # moving bar
    bx = int((t*100) % w)
    img[:, max(0,bx-5):min(w,bx+5), 2] = 255
    # encode as jpeg via PIL if available else raw not supported; require PIL
    from PIL import Image
    im = Image.fromarray(img[:,:,::-1])  # RGB
    buf = io.BytesIO()
    im.save(buf, format="JPEG", quality=70)
    return buf.getvalue()

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        path = self.path.split("?")[0]
        if path not in ("/", "/video.mjpg", "/snapshot.jpg"):
            self.send_error(404); return
        # auth
        if self.server.auth:
            import base64
            ah = self.headers.get("Authorization", "")
            if not ah.startswith("Basic "):
                self.send_response(401); self.send_header("WWW-Authenticate","Basic realm=\"cam\""); self.end_headers(); return
            dec = base64.b64decode(ah[6:]).decode()
            if dec != f"{self.server.user}:{self.server.password}":
                self.send_error(403); return
        if path == "/snapshot.jpg":
            frame = make_frame(self.server.w, self.server.h, time.time())
            self.send_response(200)
            self.send_header("Content-Type", "image/jpeg")
            self.send_header("Content-Length", str(len(frame)))
            self.end_headers()
            self.wfile.write(frame)
            return
        # stream
        self.send_response(200)
        self.send_header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
        self.end_headers()
        t0 = time.time()
        i = 0
        try:
            while True:
                if self.server.drop_every and i % self.server.drop_every == 0 and i>0:
                    i += 1; time.sleep(self.server.delay); continue
                frame = make_frame(self.server.w, self.server.h, time.time()-t0)
                self.wfile.write(b"--frame\r\nContent-Type: image/jpeg\r\n\r\n" + frame + b"\r\n")
                self.wfile.flush()
                i += 1
                time.sleep(1.0/self.server.fps + self.server.delay)
        except (BrokenPipeError, ConnectionResetError):
            pass

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8080)
    ap.add_argument("--width", type=int, default=640)
    ap.add_argument("--height", type=int, default=480)
    ap.add_argument("--fps", type=int, default=15)
    ap.add_argument("--auth", action="store_true")
    ap.add_argument("--user", default="admin")
    ap.add_argument("--password", default="secret")
    ap.add_argument("--delay", type=float, default=0.0)
    ap.add_argument("--drop-every", type=int, default=0, help="drop every Nth frame (0=none)")
    args = ap.parse_args()
    httpd = HTTPServer(("0.0.0.0", args.port), Handler)
    httpd.w = args.width; httpd.h = args.height; httpd.fps = args.fps
    httpd.auth = args.auth; httpd.user = args.user; httpd.password = args.password
    httpd.delay = args.delay; httpd.drop_every = args.drop_every
    print(f"Serving MJPEG on http://localhost:{args.port}/video.mjpg  (snapshot: /snapshot.jpg)")
    if args.auth: print(f"auth: {args.user}:{args.password}")
    httpd.serve_forever()

if __name__ == "__main__":
    main()
