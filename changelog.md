# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/)
and follows the principles of [Keep a Changelog](https://keepachangelog.com/).

---

## [Unreleased]

### Added
- 

### Changed
- 

### Deprecated
-

### Removed
-

### Fixed
-

### Security
-

---

## [0.2.0] – 2026-01-22

### Added
- Browser + OAuth authentication support (device flow, token refresh, token persistence)
- OAuth example to guide device flow setup
- Song metadata endpoint (`get_song`)

### Changed
- Client auth handling generalized (bearer vs SAPISID hash) and docs expanded

### Fixed
- OAuth token expiration handling now computed when only `expires_in` is provided

---

## [0.1.0] – 2026-01-15

### Added
- Initial public release.
- Playlists endpoints
