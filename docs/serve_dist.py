#!/usr/bin/env python3
"""Serves a built bundle the way a real static host would.

Resolves paths like GitHub Pages: `/a/b` is `a/b.html` when `route_pages.py`
wrote one, an unknown extensionless path gets `404.html` with status 404. A
bundle without `404.html` (`just build`) rewrites those to `index.html`, so a
refresh on any route still boots the app.

`python3 -m http.server` ignores the `.br` siblings `pre_compress` emits, so
the wasm goes over the wire uncompressed and a measurement against it says
nothing about compression. This serves the precompressed file when the client
asks for brotli, and gzips on the fly otherwise - Chrome only advertises `br`
over HTTPS, so a plain-http localhost run would otherwise fall back to
uncompressed and look like compression did nothing.
"""

import functools
import gzip
import os
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class Handler(SimpleHTTPRequestHandler):
    def send_head(self):
        path = self.translate_path(self.path)
        status = 200

        # The routing is client-side; anything that looks like a file is still left to 404.
        if not os.path.isfile(path) and not os.path.splitext(path)[1]:
            candidates = [os.path.join(path, "index.html")]
            if not path.endswith("/"):
                candidates.insert(0, path + ".html")
            not_found = self.translate_path("/404.html")
            found = next((c for c in candidates if os.path.isfile(c)), None)
            if found:
                path = found
            elif os.path.isfile(not_found):
                path, status = not_found, 404
            else:
                path = self.translate_path("/index.html")

        accepted = self.headers.get("Accept-Encoding", "")

        if "br" in accepted:
            try:
                return self._send_encoded(open(path + ".br", "rb").read(), "br", path, status)
            except OSError:
                pass

        encoding = "gzip" if "gzip" in accepted else None
        try:
            with open(path, "rb") as f:
                body = f.read()
        except OSError:
            return super().send_head()
        if encoding:
            body = gzip.compress(body, 6)
        return self._send_encoded(body, encoding, path, status)

    def _send_encoded(self, body, encoding, path, status=200):
        self.send_response(status)
        self.send_header("Content-Type", self.guess_type(path))
        if encoding:
            self.send_header("Content-Encoding", encoding)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        from io import BytesIO

        return BytesIO(body)


if __name__ == "__main__":
    directory = sys.argv[1]
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 8080
    handler = functools.partial(Handler, directory=directory)
    print(f"serving {directory} on http://localhost:{port}")
    ThreadingHTTPServer(("127.0.0.1", port), handler).serve_forever()
