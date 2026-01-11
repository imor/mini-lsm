<!--
  mini-lsm-book © 2022-2025 by Alex Chi Z is licensed under CC BY-NC-SA 4.0
-->

# Environment Setup

The starter code and reference solution are available at [https://github.com/skyzh/mini-lsm](https://github.com/skyzh/mini-lsm).

## Install Rust

Visit [https://rustup.rs](https://rustup.rs) and follow the instructions to install Rust.

## Clone the Repository

```
git clone https://github.com/skyzh/mini-lsm
```

## Open the Starter Code

```
cd mini-lsm/mini-lsm-starter
code .
```

## Install Required Tools

This project requires Rust version `1.74` or later. Run the following command to install the necessary tools:

```
cargo x install-tools
```

## Run Tests

```
cargo x copy-test --week 1 --day 1
cargo x scheck
```

You're ready to begin [Week 1: Mini-LSM](./week1-overview.md).

{{#include copyright.md}}
