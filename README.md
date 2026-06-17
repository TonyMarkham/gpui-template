# GPUI Template

A minimal desktop app template built with GPUI and GPUI Component.

![Image #1](docs/img/gpui_template_01.png)

## Prerequisites

- rustup, with the stable Rust toolchain installed
- Git
- Windows, macOS, or Linux
- OS-native build tools required by Rust desktop applications

Install rustup from <https://rustup.rs/>.

## Setup

```bash
git clone --recurse-submodules https://github.com/TonyMarkham/gpui-template.git
cd gpui-template
```

If you cloned without submodules, initialize them before building:

```bash
git submodule update --init --recursive
```

If the repository was moved or renamed after a build, clear stale absolute paths from Cargo's build output:

```bash
cargo clean
```

## Run

```bash
cargo run
```

## Check

```bash
cargo check
```
