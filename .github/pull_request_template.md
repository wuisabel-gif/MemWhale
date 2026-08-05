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
- [ ] `npm run build` passes
- [ ] `cargo fmt` + `cargo check` pass (in `src-tauri/`)
- [ ] Tested locally against my own MemoryWhale data

## Contribution rules checklist
- [ ] Preserves original evidence (command, args, stdout, stderr, exit code, cwd, timestamp) where the change touches capture
- [ ] No implicit cloud sync — any remote/sync behavior is explicit and documented
- [ ] Database changes use clear SQLite tables/migrations, not opaque blobs
- [ ] Still works fully offline, with no account or server
