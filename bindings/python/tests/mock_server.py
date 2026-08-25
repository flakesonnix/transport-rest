"""Tiny threaded HTTP mock implementing transport.rest-shaped responses."""

from __future__ import annotations

import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse


class _Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):  # silence test output
        pass

    def do_GET(self):
        parsed = urlparse(self.path)
        handler = self.server.routes.get(("GET", parsed.path))
        if handler is None:
            body = json.dumps({"message": f"no route for {parsed.path}"}).encode()
            self._respond(404, body)
            return
        query = {k: v[0] for k, v in parse_qs(parsed.query).items()}
        status, body, headers = handler(query)
        if isinstance(body, (dict, list)):
            body = json.dumps(body).encode()
        elif isinstance(body, str):
            body = body.encode()
        self._respond(status, body, headers or {})

    def _respond(self, status: int, body: bytes, headers: dict):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        for key, value in headers.items():
            self.send_header(key, value)
        self.end_headers()
        self.wfile.write(body)


class MockServer:
    """Context manager yielding a local server; register via ``route()``."""

    def __init__(self) -> None:
        self._server = ThreadingHTTPServer(("127.0.0.1", 0), _Handler)
        self._server.routes = {}
        self.requests: list = []
        original = _Handler.log_message

        class _Recording(_Handler):
            def log_message(self2, *args):
                pass

            def do_GET(self2):  # record then dispatch
                self.requests.append(self2.path)
                _Handler.do_GET(self2)

        self._server.RequestHandlerClass = _Recording
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)

    def route(self, path: str, handler) -> "MockServer":
        """handler(query: dict) -> (status, body_dict_or_str, headers_dict)."""
        self._server.routes[("GET", path)] = handler
        return self

    def __enter__(self) -> "MockServer":
        self._thread.start()
        return self

    def __exit__(self, *exc) -> None:
        self._server.shutdown()
        self._server.server_close()

    @property
    def url(self) -> str:
        return f"http://127.0.0.1:{self._server.server_port}"
