# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| 0.8.x (latest release on `main`) | ✅ security fixes |
| < 0.8.0 | ❌ upgrade — fixes are not backported unless noted in release notes |

(This table is updated with each release; older tags stop receiving fixes when a
new minor ships.)

## Reporting a vulnerability

Please **do not** open a public issue for security vulnerabilities.

Use one of these private channels instead:

1. **GitHub private vulnerability reporting** — open the repository's
   **Advisories** page (Security → Advisories) and select **Report a
   vulnerability**. This is the preferred channel.
2. If private reporting is unavailable, contact the maintainers through a
   private channel listed on their GitHub profiles.

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
