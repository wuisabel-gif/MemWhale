# Security Policy

## Supported versions

Security fixes are applied to the latest release on the default branch. Older tagged releases may not receive backports unless noted in the release notes.

## Reporting a vulnerability

Please **do not** open a public issue for security vulnerabilities.

Use one of these private channels instead:

1. **GitHub private vulnerability reporting** — open a security advisory from the repository **Security** tab (preferred when enabled).
2. If private reporting is unavailable, contact the maintainers through a private channel listed on their GitHub profiles.

### What to include

- Affected version or commit
- A minimal reproduction (steps or a small PoC)
- Impact assessment (confidentiality / integrity / availability)

### What not to include publicly

- Captured secrets, tokens, or credentials
- Full terminal dumps that may contain private local data
- Unredacted user paths or environment variables that expose secrets

## Coordinated disclosure

We aim to acknowledge valid reports promptly and coordinate a fix before any public disclosure. Please give maintainers a reasonable window to investigate and release a fix before publishing details.

## Local data threat model

MemoryWhale stores sensitive development evidence on the local machine. For how local data is protected (threat model, not vulnerability reporting), see [docs/SECURITY.md](docs/SECURITY.md).
