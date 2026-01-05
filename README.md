# Conway's Game of Rust

> A (soon-to-be) high-performance, terminal-based implementation of Conway's Game of Life, built with **Rust** and **Ratatui**.

**Conway's Game of Rust** is a TUI (Terminal User Interface) application that brings the popular classic cellular automata game "Conway's Game of Life" to the terminal! It features a modal interface inspired by Vim (Normal, Visual, and Insert/Running modes), allowing for pattern editing and simulation control.

Built for Linux/Unix systems with primary support for **NixOS** via Flakes.

---

## Features

* **Modal Editing**: Distinct modes for navigation, selection, and simulation, inspired by modal editors like Vim.
* **Visual Mode**: Select and toggle large regions of cells simultaneously using a visual anchor system.
* **Vim-Key Navigation**: Full support for `h`, `j`, `k`, `l` movement.
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

## 🎮 Usage

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
| `q` | Quit Application | All Modes |

---

## Roadmap & Engineering Goals

The current version (v0.1.0) focuses on a stable, idiomatic implementation of the core Game of Life rules using a fixed-size vector grid. Future updates will focus on scalability and rendering optimizations.

### Upcoming Features

* [ ] **Camera Controls**:
* **Zooming**: Implement sub-cell rendering (using Unicode block characters like `▀` and `█`) to display 2x or 4x more cells per terminal character.
* **Panning**: Decouple the viewport from the grid memory, allowing the user to scroll across an infinite or massive plane.


* [ ] **Performance Optimizations**:
* Currently, the engine uses a dense 2D vector, which limits performance on extremely large grids.
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
