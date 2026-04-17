# Yui

Yui is a high-performance, aesthetically pleasing Pomodoro timer for your terminal. Built with Rust and Ratatui, it features a unique "Chaos-Bar" progress indicator and desktop notifications to keep you in the flow.

## Features

- **Focus Sessions**: Optimized Pomodoro workflow with Work, Short Break, and Long Break phases.
- **Dynamic TUI**: A beautiful, responsive terminal interface with custom RGB color palettes.
- **Chaos Bar**: A unique progress animation that lives and breathes as you work.
- **Live Settings**: Adjust your session lengths and notification modes on the fly without restarting.
- **Desktop Notifications**: Integrated system notifications to keep you informed even when the terminal is hidden.
- **Keyboard Centric**: Full control via intuitive keybindings.

## Keybindings

### Timer Screen
| Key | Action |
| :--- | :--- |
| `Space` | Toggle Pause / Resume |
| `Enter` | Advance to next phase (when session is done) |
| `s` | Skip current phase |
| `r` | Reset current timer |
| `t` | Open Settings |
| `q` | Quit |

### Settings Screen
| Key | Action |
| :--- | :--- |
| `Up` / `Down` | Navigate settings |
| `Left` / `Right` | Adjust value (minutes) |
| `H` / `L` | Adjust value by 5 minutes |
| `Enter` / `t` | Save and return to Timer |

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Build from source
```bash
git clone https://github.com/swayam-rajput/pomo-tui.git
cd pomo-tui
cargo build --release
```

The binary will be available at `./target/release/rez`.

## Configuration

You can customize the following session lengths directly in the app:
- **Work Session**: 1-99 minutes
- **Short Break**: 1-99 minutes
- **Long Break**: 1-99 minutes
- **Notification Modes**: Off, Work Only, Break Only, or All.

## License

This project is licensed under the MIT License.

---
Built by [swayam-rajput](https://github.com/swayam-rajput)
