# DMARC Report Viewer
A lightweight selfhosted standalone DMARC report viewer that fetches input data periodically from an IMAP mailbox.

Ideal for smaller selfhosted mailservers.
The application is a single executable written in Rust.
It combines the DMARC report parser with an IMAP client and an HTTP server for easy access of the reports.
You can run the executable directly on any Linux, Windows or MacOS system,
or just use the lightweight Linux Docker image to deploy the application.

## Run with Docker
The latest version is always published automatically as Docker image in the GitHub container registry.
You can fetch it using the command `docker pull ghcr.io/cry-inc/dmarc-report-viewer`.

To show all configuration parameters with the possible environment variables run the command
`docker run --rm ghcr.io/cry-inc/dmarc-report-viewer ./dmarc-report-viewer --help`.

You can configure the application using command line arguments or enviroment variables.
For the Docker use case environment variables are recommended.
Do not forget to expose the port for the HTTP server!
Since the application has no persistent data no volumes are required.
Here is an complete example: 

    docker run --rm \
      -e IMAP_HOST=mymailserver.com \
      -e IMAP_USER=dmarc@mymailserver.com \
      -e IMAP_PASSWORD=mysecurepassword \
      -e HTTP_SERVER_PORT=8123 \
      -e HTTP_SERVER_USER=webui-user \
      -e HTTP_SERVER_PASSWORD=webui-password \
      -p 8123:8123 \
      ghcr.io/cry-inc/dmarc-report-viewer`

## Build from Source
1. Install Rust (see https://rustup.rs/)
2. Check out this repository or download and extract the ZIP
3. Run the command `cargo build --release` in the folder with this README file
4. Find the compiled executable in the folder `target/release`
5. Use the help argument to list all possible configuration parameters: `dmarc-report-viewer --help`

## Acknowledgements
- https://github.com/bbustin/dmarc_aggregate_parser was used as foundation for my slightly modified DMARC report parser
- https://github.com/chartjs/Chart.js is embedded as JavaScript library for generating nice charts
