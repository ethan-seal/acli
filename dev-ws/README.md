Temporary dev workspace for anki-wrapper real-anki exploration

Steps
1) Populate the local Anki subrepo:
   cd anki-wrapper/anki
   git clone --depth=1 https://github.com/ankitects/anki .

2) From this directory, build/run with the real backend:
   cargo run -p anki-wrapper --features real-anki --example explore

Notes
- This workspace only includes `anki-wrapper` to avoid conflicts with the main workspace.
- The `anki-wrapper` crate has a path dependency to `anki-wrapper/anki/rslib` gated by the `real-anki` feature.
- Some platforms may need recent stable Rust. If you see toolchain errors, run `rustup default stable`.

