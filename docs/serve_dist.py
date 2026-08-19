#!/usr/bin/env python3
"""Serves a built bundle the way a real static host would.

`python3 -m http.server` ignores the `.br` siblings `pre_compress` emits, so
the wasm goes over the wire uncompressed and a measurement against it says
nothing about compression. This serves the precompressed file when the client
asks for brotli, and gzips on the fly otherwise - Chrome only advertises `br`
over HTTPS, so a plain-http localhost run would otherwise fall back to
uncompressed and look like compression did nothing.
"""

import functools
import gzip
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class Handler(SimpleHTTPRequestHandler):
    def send_head(self):
        path = self.translate_path(self.path)
        accepted = self.headers.get("Accept-Encoding", "")

        if "br" in accepted:
            try:
                return self._send_encoded(open(path + ".br", "rb").read(), "br", path)
            except OSError:
                pass

        if "gzip" in accepted:
            try:
                with open(path, "rb") as f:
                    body = gzip.compress(f.read(), 6)
                return self._send_encoded(body, "gzip", path)
            except OSError:
                pass

        return super().send_head()

    def _send_encoded(self, body, encoding, path):
        self.send_response(200)
        self.send_header("Content-Type", self.guess_type(path))
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
