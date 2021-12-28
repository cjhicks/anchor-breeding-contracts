# anchor-breeding-contracts
Repo for our custom breeding contracts using https://github.com/project-serum/anchor

## Hello World Rust example

### Create project:
```bash
cargo new hello_world

# Output: Created binary (application) `hello_world` package
```

Should see the main fn in `main.rs`
```rust
fn main() {
    println!("Hello, world!");
}
```

Manifest (`cargo.toml`) should look like this:
```
[package]
name = "hello_world"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
```

### Compile and Run:
```bash
cd hello_world

cargo build

cargo run

# output: Hello, world!
```
