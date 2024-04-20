# DMARC Report Viewer
[![Build Status](https://github.com/cry-inc/dmarc-report-viewer/workflows/CI/badge.svg)](https://github.com/cry-inc/dmarc-report-viewer/actions)
[![No Unsafe](https://img.shields.io/badge/unsafe-forbidden-brightgreen.svg)](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Dependencies](https://deps.rs/repo/github/cry-inc/dmarc-report-viewer/status.svg)](https://deps.rs/repo/github/cry-inc/dmarc-report-viewer)

A lightweight selfhosted standalone DMARC report viewer that automatically fetches input data periodically from an IMAP mailbox.

Ideal for smaller selfhosted mailservers.
The application is a single executable written in Rust.
It combines the DMARC report parser with an IMAP client and an HTTP server for easy access of the reports.
You can run the executable directly on any Linux, Windows or MacOS system.
Alternatively, you can use the tiny Linux Docker image to deploy the application.

## Features
- [x] Lightweight Docker image for easy deployment
- [x] Secure IMAP client
- [x] Automatic fetching of reports from IMAP inbox
- [x] Robust parsing of XML DMARC reports
- [x] Embedded HTTP server for UI
- [x] Basic Auth password protection for HTTP server
- [x] Easy configuration via command line arguments or ENV variables
- [x] Summary with diagrams for domains, organizations and passed/failed checks
- [x] Automatic HTTPS via ACME/Let's Encrypt
- [ ] Viewing of individual DMARC reports
- [ ] Viewing filtered lists of reports
- [ ] Viewing parsing errors for XML DMARC reports

## Run with Docker
The latest version is always published automatically as Docker image in the GitHub container registry.
You can download the image using the command `sudo docker pull ghcr.io/cry-inc/dmarc-report-viewer`.

List all available configuration parameters with the corresponding environment variables by running this command:
`sudo docker run --rm ghcr.io/cry-inc/dmarc-report-viewer ./dmarc-report-viewer --help`.

You can configure the application with command line arguments or environment variables.
For the Docker use case, environment variables are recommended.
Do not forget to forward the port for the HTTP server!

Here is a concrete example: 

    sudo docker run --rm \
      -e IMAP_HOST=imap.mymailserver.com \
      -e IMAP_USER=dmarc@mymailserver.com \
      -e IMAP_PASSWORD=mysecurepassword \
      -e HTTP_SERVER_PORT=8123 \
      -e HTTP_SERVER_USER=webui-user \
      -e HTTP_SERVER_PASSWORD=webui-password \
      -p 8123:8123 \
      ghcr.io/cry-inc/dmarc-report-viewer

### HTTPS
By default, the application will start an unencrypted and unsecure HTTP server.
It is *strongly* recommended use the automatic HTTPS feature that will automatically fetch and renew a certificate from Let's Encrypt.
This feature uses the TLS-ALPN-01 challenge, which uses the HTTPS port 443 also for the challenge. No port 80 required!
Alternatively, you can use an separate HTTPS reverse proxy like [Caddy](https://caddyserver.com/) to secure it.

To use the automatic HTTPS feature you need to make sure that the public port exposed to the internet is 443.
You should also persist the certificate caching directory on your host file system:

    sudo docker run --rm \
      -e IMAP_HOST=imap.mymailserver.com \
      -e IMAP_USER=dmarc@mymailserver.com \
      -e IMAP_PASSWORD=mysecurepassword \
      -e HTTP_SERVER_PORT=8443 \
      -e HTTP_SERVER_USER=webui-user \
      -e HTTP_SERVER_PASSWORD=webui-password \
      -e HTTPS_AUTO_CERT=true \
      -e HTTPS_AUTO_CERT_CACHE=/certs \
      -e HTTPS_AUTO_CERT_MAIL=admin@mymailserver.com \
      -e HTTPS_AUTO_CERT_DOMAIN=dmarc.mymailserver.com \
      -v /host/cert/folder:/certs \
      -p 443:8443 \
      ghcr.io/cry-inc/dmarc-report-viewer

## Build from Source
1. Install Rust (see https://rustup.rs/)
2. Check out this repository or download and extract the ZIP
3. Run the command `cargo build --release` in the folder with this README file
4. Find the compiled executable in the folder `target/release`
5. Use the help argument to list all possible configuration parameters: `dmarc-report-viewer --help`

## Acknowledgements
- https://github.com/bbustin/dmarc_aggregate_parser was used as foundation for the slightly modified DMARC report parser
- https://github.com/chartjs/Chart.js is embedded as JavaScript library for generating nice charts
- See also `Cargo.toml` for other Rust dependencies that make this application possible!
