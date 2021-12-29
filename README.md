# anchor-breeding-contracts
Repo for our custom breeding contracts using https://github.com/project-serum/anchor

## Quickstart
```bash
cd breeding_cooldown

anchor build

anchor test
```

## Hello World Rust example

### Create project:
```bash
cargo new hello_world

# Output: Created binary (application) `hello_world` package
```

Should see the main fn in `hello_world/src/main.rs`
```rust
fn main() {
    println!("Hello, world!");
}
```

Manifest (`hello_world/cargo.toml`) should look like this:
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

## Anchor Hello World example

### Create Project:
```
anchor init hello_world
```

You should see an initialize instruction in `hello_world/programs/hello_world/src/lib.rs`
```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod hello_world {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
```

### Compile and Test:
```bash
cd hello_world

anchor build

anchor test

# output: ✔ Is initialized! (401ms)
```

Full CLI (for deploying, upgrading, etc) available at https://project-serum.github.io/anchor/cli/commands.html
