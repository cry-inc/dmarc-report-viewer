# Changelog

All notable changes to this project will be documented in this file.

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
