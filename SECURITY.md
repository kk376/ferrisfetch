# Security Policy

## Supported Versions

Security fixes are provided for the current release version of KKFetch.

| Version | Supported          |
| :---    | :---               |
| 0.11.x  | :white_check_mark: |
| < 0.11.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability or potential issue in KKFetch, please disclose it responsibly. Do not report security vulnerabilities via public GitHub issues.

### How to report

1. Use GitHub's private vulnerability reporting feature:
   - Navigate to the **Security** tab of the KKFetch repository (`https://github.com/kk376/kkfetch`).
   - Select **Advisories** and click **Report a vulnerability**.
2. Alternatively, contact the maintainer Kushagra Kumar (kk376) directly via GitHub or email.

### What to include

Please provide:
- A description of the issue and potential security impact.
- Steps or a minimal test case to reproduce the behavior.
- The version of KKFetch and the operating system environment (distribution, kernel, architecture) used.

### Response timeline

- **Initial confirmation**: Within 48 hours of receipt.
- **Assessment and status updates**: Regular updates while investigating and drafting a fix.
- **Coordinated disclosure**: A patch and security release will be published alongside an advisory acknowledging the reporter (unless you prefer anonymity).

## Plugin System Threat Model

- Plugins execute shell commands via `/bin/sh -c`
- Mitigations: elevated context blocked, file ownership validation, symlink rejection
- Risk: malicious config.toml can execute arbitrary commands as current user
- Recommendation: review plugin configs, use `--no-plugins` in automated contexts
