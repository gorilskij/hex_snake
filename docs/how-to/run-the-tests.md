# Run the tests and the benchmark

What they cover: [tests](../reference/testing.md).

```
cargo test                                               # ~0.2 s once built; baseline 69 passed, 1 ignored
cargo test --test docs                                   # the docs check
cargo test --release benchmark -- --ignored --nocapture  # the planning benchmark
```

- The benchmark prints planning time apart from the rest of the world update, per tick, for 24
  snakes on an 80 × 50 board with 80 apples. Compare against the numbers in
  [autopilot](../reference/autopilot.md#cost) after touching the pathfinder or plans.
- Shaders and the look are not tested: run the game ([build the web version](build-the-web-version.md)
  for the browser).
- Format before committing: `cargo fmt` (nightly rustfmt, `rustfmt.toml`). A large
  formatting-only commit goes into `.git-blame-ignore-revs`.
