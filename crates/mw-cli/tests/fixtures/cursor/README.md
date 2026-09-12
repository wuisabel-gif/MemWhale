# Cursor hook fixtures

These are synthetic, documentation-derived fixtures for the local Cursor
Agent `Shell` surface. They are **not live Cursor captures** and the
`cursor_version: "documentation-fixture"` value is deliberately not a product
version or a compatibility certification.

Reference: <https://cursor.com/docs/hooks>, inspected September 11–12, 2026.
The relevant contract is `postToolUse` with JSON-stringified `tool_output`,
and `postToolUseFailure` with failure metadata. The adapter deliberately ignores
`afterShellExecution` to avoid subscribing to overlapping execution surfaces.

Cases cover an explicit zero/nonzero exit, an unknown exit, timeout, permission
denial, interruption, and malformed result JSON. Tests substitute a real
sandbox cwd for `/fixture/project`; they never execute the commands embedded
in these payloads. Additional tests exercise bounds, redaction, policy
exclusions, repeated legitimate commands, and retrieval by fresh CLI/MCP
processes.

Live verification remains pending until someone records the actual local
Cursor version and approved hook behavior. Do not replace missing fields with
assumed values or describe this fixture suite as a live client test.
