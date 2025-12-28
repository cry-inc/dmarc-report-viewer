# Changelog

All notable changes to this project will be documented in this file.

## [2.3.1] - 2025-12-28
* Fixed issue #70 with incomplete command line help caused by new Docker health check feature
* Added failure filters to DMARC report details (implemented by @manufant)
* Updated Cargo dependencies

## [2.3.0] - 2025-11-13
* Push new Docker images also to Docker Hub (see https://hub.docker.com/r/cryinc/dmarc-report-viewer)
* Publish {major} and {major}.{minor} Docker tags for releases in addition to {major}.{minor}.{patch}
* New filtering flag for DMARC reports when DKIM and SPF policies failed (implemented by @manufant)
* Added new HTTP health check endpoint `/health`
* Added health check Docker integration
* Updated Cargo dependencies

## [2.2.2] - 2025-10-18
* Reduced default IMAP chunk size to make the configuration work out of the box with the Stalwart IMAP server (see issue #61).
  This means for inboxes with many mails (>1000) it might be now a little bit slower while updating.
  You can manually override the new default value with the command line argument `--imap-chunk-size`
  or the ENV variable `IMAP_CHUNK_SIZE` and set it back to 2000 or a even higher value.
* Improved IMAP fetching error messages to hint at incomptible IMAP chunk size that might need to be lowered in some cases
* Allow non-standard disposition value `pass` in some DMARC reports (implemented by @SuperSandro2000 in PR #59, thank you!)
* Updated Cargo dependencies

## [2.2.1] - 2025-09-02
* Updated Cargo dependencies to fix CVE-2025-58160 / GHSA-xwfj-jgwm-7wp5

## [2.2.0] - 2025-08-22
* Added DNS hostnames to IPs in sources list
* Added new precompiled Windows binary for 64bit ARM
* Speed up DNS queries by improving caching and switching to a new minimal async DNS client
* Fixed issue #51 with surgemail IMAP server by adding a client workaround
* Fixed issue #54 with duplicates for case-sensitive domains and mail addresses
* Fixed issue #53 by grouping smaller domains and organizations in pie charts on dashboard
* Updated Cargo dependencies

## [2.1.0] - 2025-07-28
* Implemented new ranked list that shows all sources and IPs for the different report types and domains
* Implemented optional customizable HTTP web hook that is called for every new mail (see `--help` for more details)
* Fixed URL for WHOIS links to use relative instead of absolute path (see bug report #50)
* Minor help and documention improvements
* Updated Cargo dependencies

## [2.0.0] - 2025-06-28
* Added support for SMTP TLS reports (implemented by @marvinruder, thank you!)
* Added support for multiple separate inboxes for DMARC and SMTP TLS reports (also implemented by @marvinruder)
* Introduced new unique IDs for reports and mails to keep URLs short
* A lot of other minor improvements and fixes
* Updated Rust Edition to 2024
* Updated Cargo and JavaScript dependencies

**Update Notes**: No configuration changes needed, it should continue to work as before.
You only need to adjust the configuration if you want to use separate mailbox folders for DMARC and SMTP TLS reports.
By default it checks the already configured IMAP mailbox for both types of reports!
See `--help` for a list of all possible configuration options and values.

## [1.8.0] - 2025-05-20
* Dashboard UI: Add filtering for charts by time span
* Dashboard UI: Add filtering for charts by domain
* Add support for attachments with uncompressed XML files
* Allow scheduling IMAP updates using cron expressions instead of intervals
* Allow SPF result "hardfail" as alias for "fail"
* Improved visualization of dynamically queried source IP properties
* Updated default IMAP chunk size to make MS Exchange servers happy
* Fix to sum up results on dashboard correctly with row count
* Fix to treat same XML file from different mails as separate XML files
* Fix to deal with file names in headers that are split into multiple parts
* Updated Cargo dependencies

## [1.7.0] - 2025-04-12
* Dashboard UI: Use fixed colors for some well known big organizations
* Dashboard UI: Limit size of legends in charts
* Dashboard UI: Made order of values in charts stable
* Extended low level logging for mail fetching and XML extraction
* Fixed embedded documentation for certificate input file
* Convert non-fatal IMAP error when closing connection into warning
* Updated Cargo dependencies

## [1.6.0] - 2025-03-20
* Improved active state of navbar links to include child pages
* Introduced separate problem flags for DKIM and SPF
* Detect more ZIP attachments correctly
* Updated Cargo dependencies, including `zip` to fix CVE-2025-29787 and `ring` to fix GHSA-4p46-pwfr-66x6

## [1.5.0] - 2025-03-02
* Fixed detection of (G)ZIP XML attachments with content type `application/octet-stream`
* Added feature to look up DNS name of Source IP
* Added feature to look up location of Source IP
  (uses free IP Geolocation API by ip-api.com, limited to 45 req/min)
* Added feature to look up Whois record of Source IP
* Updated Cargo dependencies

## [1.4.0] - 2025-02-15
* Added option to inject additional custom CA certificates
* Added option to disable TLS encryption for IMAP client
* Updated Cargo dependencies

## [1.3.0] - 2025-01-21
* Increased default IMAP check interval to 30 minutes
* More robust mail fetching (RFC822.SIZE property is now optional)
* Updated Cargo dependencies
* Allow empty `sp` field in reports instead of failing to parse whole report
* Docker images now expose port 8080 for improved auto-discovery
* Made Web UI responsive to also work on smaller screens

## [1.2.0] - 2025-01-04
* Fixed bugs and improved E-Mail subject decoding
* Added Linux ARM 64bit binary artifacts and restructured builds
* Added support for ARM 64bit Linux Docker images and publish them to Github registry
* Updated Cargo dependencies

## [1.1.2] - 2025-01-01
* Fix issue with iCloud Mail server not returning the mail body
* Improved log messages for mails without XML report data
* Extended log messages with time needed for background updates

## [1.1.1] - 2024-12-31
* Some minor UI styling improvements and fixes
* Fixed XML count bug in mails table
* Better subject shortening for mails table
* Updated Cargo dependencies
* Added embedded help for some of the harder to understand policy fields in reports

## [1.1.0] - 2024-12-23
* Restyled the whole application to look a bit nicer
* Fixed missing git info (commit hash and ref name) in Docker builds
* Added Mac OS builds for CI and Releases
* Updated Cargo dependencies

## [1.0.0] - 2024-12-20
First stable release.
