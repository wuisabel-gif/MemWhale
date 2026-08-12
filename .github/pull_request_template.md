## What this changes
<!-- One or two sentences. What does this PR do? -->

## Why it matters
<!-- Connect back to the core problem: terminal history and debugging context
disappear too easily. Which value does this serve? -->
Closes #

## What's in it
<!-- Bullet the files/areas touched. -->
-

## Verification
<!-- How you know it works. Paste output/screenshots where useful. -->
- [ ] `npm test` passes when frontend code changes
- [ ] `npm run build` passes when frontend code changes
- [ ] `cargo fmt --all -- --check` passes when Rust code changes
- [ ] `cargo clippy -p memorywhale-core -p memorywhale-cli --all-targets -- -D warnings` passes when Rust code changes
- [ ] `cargo test -p memorywhale-core -p memorywhale-cli` passes when Rust code changes
- [ ] `cargo build --workspace` passes for workspace changes
- [ ] Tested locally against my own MemoryWhale data

## Contribution rules checklist
- [ ] Preserves original evidence (command, args, stdout, stderr, exit code, cwd, timestamp) where the change touches capture
- [ ] No implicit cloud sync — any remote/sync behavior is explicit and documented
- [ ] Database changes use clear SQLite tables/migrations, not opaque blobs
- [ ] Still works fully offline, with no account or server
