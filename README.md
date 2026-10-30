# Conway's Game of Rust

> A (soon-to-be) high-performance, terminal-based implementation of Conway's Game of Life, built with **Rust** and **Ratatui**.

**Conway's Game of Rust** is a TUI (Terminal User Interface) application that brings the popular classic cellular automata game "Conway's Game of Life" to the terminal! It features a modal interface inspired by Vim (Normal, Visual, and Insert/Running modes), allowing for pattern editing and simulation control.

Built for Linux/Unix systems with primary support for **NixOS** via Flakes.

---

## Features

* **Modal Editing**: Distinct modes for navigation, selection, and simulation, inspired by modal editors like Vim.
* **Visual Mode**: Select and toggle large regions of cells simultaneously using a visual anchor system.
* **Vim-Key Navigation**: Full support for `h`, `j`, `k`, `l` movement, including repetition like `10j`.
* **Pause & Resume**: Stop the simulation at any time to modify the grid state manually.
* **Reproducible Builds**: Fully flake-enabled for deterministic builds on Nix systems.

---

## Installation

### Option 1: Using Cargo (Universal)

If you have the Rust toolchain installed, you can build directly from crates.io (once published) or from source:

```bash
# From source
git clone https://github.com/Shonhh/Conways-Game-of-Rust.git
cd Conways-Game-of-Rust
cargo install --path .

```

### Option 2: Using Nix (Recommended)

This project features a hermetic development environment and build process using **Nix Flakes**.

**Run directly without installing:**

```bash
nix run github:Shonhh/Conways-Game-of-Rust

```

**Build the binary:**

```bash
nix build
# Binary is available at ./result/bin/conway_game_of_rust

```

**Enter the development shell:**

```bash
nix develop

```

---

## Usage

Run the application from your terminal:

```bash
conway_game_of_rust

```

### Key Bindings

| Key | Action | Context |
| --- | --- | --- |
| **Movement** |  |  |
| `h` / `←` | Move Cursor Left | Normal / Visual |
| `j` / `↓` | Move Cursor Down | Normal / Visual |
| `k` / `↑` | Move Cursor Up | Normal / Visual |
| `l` / `→` | Move Cursor Right | Normal / Visual |
| **Control** |  |  |
| `Enter` | Play / Pause Simulation | All Modes |
| `Space` | Toggle Cell State | Normal Mode |
| `Space` | Toggle Selection | Visual Mode |
| `v` | Enter **Visual Mode** | Normal Mode |
| `Esc` | Return to **Normal Mode** | Visual Mode |
| `r` | Reset / Clear Grid | Normal / Visual |
| `[` / `]` | Decrement / Increment Time Between Generations | All Modes |
| `q` | Quit Application | All Modes |

---

## Roadmap & Engineering Goals

The current version (v0.1.0) focuses on a stable, idiomatic implementation of the core Game of Life rules using a fixed-size vector grid. Future updates will focus on scalability and rendering optimizations.

### Upcoming Features

* [ ] **Camera Controls**:
* **Zooming**: Implement sub-cell rendering (using Unicode block characters like `▀` and `█`) to display 2x or 4x more cells per terminal character.
* **Panning**: Decouple the viewport from the grid memory, allowing the user to scroll across an infinite or massive plane.


* [ ] **Performance Optimizations**:
* Currently, the engine uses a dense, flattened vector, which limits performance on extremely large grids.
* **Goal**: Refactor to a **sparse matrix** or **HashLife** algorithm to support millions of active cells with minimal memory overhead.


* [ ] **Larger/Infinite Grid**:
* Transition from fixed `128x80` bounds to a dynamically resizing vector or chunk-based system to support larger patterns like "glider guns" and "logic gates."

* [ ] **Templates/RLE code implementation**:
* Add cool templates that showcase cool aspects of Conway's Game of Life.
* Allow for users to utilize RLE codes that exist in previous implementations of this game, letting users copy and publish creations.

* [ ] **Step-by-Step Control**:
* Fine-grained control over the simulation speed and state, like stepping or slowing down simulation speed.


---

## Technical Details

* **Language**: Rust (2021 Edition)
* **TUI Framework**: [Ratatui](https://github.com/ratatui-org/ratatui)
* **Event Handling**: Crossterm
* **Build System**: Nix Flakes + Cargo

## License

This project is licensed under the **MIT License**. See the `LICENSE` file for details.

## Headless experiments

Run the same simulation engine without terminal rendering or a frame delay:

```bash
# A centered glider on a 64 by 48 grid; CSV goes to stdout.
cargo run --release -- --headless --width 64 --height 48 \
  --pattern glider --generations 100

# Reproducible random initialization; output must be a new file.
cargo run --release -- --headless --width 512 --height 512 \
  --seed 42 --density 0.3 --generations 1000 --output experiment.csv

cargo run -- --help
```

Running with no arguments still opens the existing terminal interface. With Nix,
pass the same arguments after `nix run . --`.

Available patterns are `empty`, `block`, `blinker`, and `glider`. Patterns are
centered using integer division; a pattern that does not fit is rejected rather
than clipped. `--seed` selects random initialization instead of a built-in
pattern, and cannot be combined with `--pattern`. The default density is 0.3;
`--density` requires `--seed`. Seeds are unsigned 64-bit integers, including zero.
Random initialization uses SplitMix64 in row-major order, converting its upper
53 bits to a probability in [0, 1). The same dimensions, seed, density, and
number of generations reproduce the same states, independent of timing.

The grid uses **fixed, dead boundaries**: every cell outside it is dead. There
is no wrapping or automatic expansion. A glider eventually reaches the edge,
so choose enough space for the number of generations you want to observe.

CSV contains these columns:

| Column | Meaning |
| --- | --- |
| `width`, `height` | Grid dimensions in cells |
| `boundary` | `fixed-dead` |
| `initialization` | Pattern name or `random` |
| `seed`, `density` | Random configuration; blank for built-in patterns |
| `generation` | Zero for the initial state, then one row per update |
| `elapsed_update_ns` | Cumulative time spent inside `next_generation()`, in nanoseconds |
| `live_cells` | Population after this generation |
| `checksum` | 16-digit hexadecimal FNV-1a state checksum |

The checksum covers width and height as little-endian u64 values, followed by
row-major cell bytes (dead = 0, alive = 1). It is a reproducibility check, not a
cryptographic hash or a substitute for comparing complete states in tests.

Timing excludes initialization, population counting, checksums, and CSV output.
Those operations still scan the grid and affect total runtime and cache state.
These timings are an experiment log, not a controlled benchmark; use a dedicated
benchmark before making performance claims. Output defaults to stdout; `--output -`
also selects stdout. A named output file is created only if it does not already
exist, and is explicitly flushed so write errors are reported.

## Correctness checks

```bash
cargo test
```

Tests cover all 512 combinations of center-cell state and eight neighbors,
block stability, blinker oscillation, glider translation, dead boundaries,
invalid coordinates, tiny grids, rectangle editing, dimension validation,
checksums, CLI argument validation, random initialization, and CSV output.

`Grid` stores dimensions privately and exposes `width()` and `height()`. Both
coordinates are validated before calculating a flat index. Invalid `get()`
returns `None`; invalid `set()` and `toggle_cell()` do nothing. Constructors
reject zero dimensions and dimension multiplication overflow.
