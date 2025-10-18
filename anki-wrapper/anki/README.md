This directory is intended to hold a local checkout of the Anki repository for developing and exploring the `real-anki` backend.

Populate it by cloning Anki here:

    git clone --depth=1 https://github.com/ankitects/anki .

The `anki` Rust crate lives at `rslib/`. The `anki-wrapper` crate references it via a path dependency when the `real-anki` feature is enabled.

Notes
- The Anki repo is a Cargo workspace; building the `anki` crate requires the whole workspace context.
- Use a temporary local workspace that includes both `anki-wrapper` and `anki` if you want to compile with `--features real-anki` without impacting the parent workspace.
- Some platform or feature combinations may require a recent stable toolchain.
