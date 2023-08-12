# Arweave tools (rust)
This is port https://github.com/virdpool/arweave_tools to Rust

# Tech debt

* `cargo test` and `cargo check --tests` doesn't show correctly unused functions (fn decode(&self) marked as unused, but used in tests)
