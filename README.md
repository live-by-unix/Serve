Serve — A Lightweight Static HTTP/HTTPS File Hoster

Serve is a tiny, fast, zero‑config static file hoster written in Rust. It is designed to behave like a modern, optimized version of "python3 -m http.server" but with HTTPS support, directory listings, index.html handling, and clean logging.

Serve hosts any directory over HTTP or HTTPS with a single command.

Features

- Serve any directory over HTTP or HTTPS
- Default port 8000 if none is provided
- Automatic index.html serving
- Optional -i flag to ignore index.html and show directory listing
- Clean HTML directory listings
- Self‑signed certificate generation for HTTPS
- ANSI‑colored logging for requests, status codes, and file serving
- Simple, predictable CLI
- Zero configuration, zero dependencies beyond the binary

Installation

Build from source:

    cargo build --release

The binary will be located at:

    target/release/serve

Usage

Basic HTTP hosting:

    serve .

HTTP on a specific port:

    serve . 5000 --http

HTTPS hosting:

    serve . 5000 --https

Ignore index.html and show directory listing:

    serve . 8000 -i

Show version:

    serve --version

Show help:

    serve --help

Behavior

Serve always hosts the directory provided as the first argument.

If index.html exists:
- It is served automatically unless -i is used.

If index.html does not exist:
- A directory listing is generated.

If no port is provided:
- Serve defaults to port 8000.

If no protocol flag is provided:
- Serve defaults to HTTP.

HTTPS certificates:
- Stored in ~/.serve/
- Auto‑generated on first HTTPS run

Examples

Serve the current directory on port 8000:

    serve .

Serve a folder on port 3000:

    serve public 3000

Serve with HTTPS:

    serve site 8443 --https

Ignore index.html:

    serve . -i

License
BSD 3 LICENSE. 
