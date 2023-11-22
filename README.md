### 1. Install Shuttle:

```bash
cargo binstall cargo-shuttle
```

### 2. Sign in to Shuttle:

```bash
cargo shuttle login
```

### 3. Run the code:

```bash
cargo shuttle run

# This will e(x)ecute `cargo shuttle run` when you save a file.
cargo watch -x 'shuttle run'
# This will also (q)uietly (c)lear the console between runs.
cargo watch -qcx 'shuttle run'
```
