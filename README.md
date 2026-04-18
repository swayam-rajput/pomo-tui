# Pomo-TUI
![tui](https://raw.githubusercontent.com/swayam-rajput/portfolio/refs/heads/main/public/pomo-tui.png)
Pomo-TUI is a high-performance, aesthetically pleasing Pomodoro timer for your terminal. Built with Rust and Ratatui, it features a unique "Chaos Bar" progress indicator, persistent settings, and desktop notifications to keep you in the flow.

---

## Features

* Focus Sessions: Full Pomodoro workflow with Work, Short Break, and Long Break phases
* Auto Advance Mode: Seamless transitions between sessions without manual input
* Dynamic TUI: Responsive terminal UI with custom RGB color palette
* Chaos Bar: Animated progress indicator that evolves as time passes
* Live Settings: Adjust durations, notifications, and behavior in real-time
* Persistent Configuration: Settings are automatically saved and restored
* Desktop Notifications: Get notified when sessions complete
* Keyboard Centric: Fast, intuitive keybindings

---

## Keybindings

### Timer Screen

| Key     | Action                              |
| :------ | :---------------------------------- |
| `Space` | Toggle Pause / Resume               |
| `Enter` | Advance to next phase (manual mode) |
| `s`     | Skip current phase                  |
| `r`     | Reset current timer                 |
| `t`     | Open Settings                       |
| `q`     | Quit                                |

---

### Settings Screen

| Key              | Action                    |
| :--------------- | :------------------------ |
| `Up` / `Down`    | Navigate settings         |
| `Left` / `Right` | Adjust value              |
| `H` / `L`        | Adjust value by 5 minutes |
| `Enter` / `t`    | Save and return           |

---

## Installation

### Prerequisites

* Rust (latest stable)

### Build from source

```bash
git clone https://github.com/swayam-rajput/pomo-tui.git
cd pomo-tui
cargo build --release
```

Binary:

```bash
./target/release/pomo-tui
```

---

## Configuration

Settings are managed inside the app and automatically saved to:

```text
settings.json
```

### Configurable Options

* Work Duration (1–99 minutes)
* Short Break Duration
* Long Break Duration
* Notification Mode:

  * Off
  * Work Only
  * Break Only
  * All
* Auto Advance:

  * On → transitions automatically
  * Off → manual control

---

## Notifications

Uses notify-rust for desktop notifications.

* Works best on Linux
* May require additional setup on Windows/macOS
* Sound is handled separately (if enabled)

---

## Tech Stack

* Rust
* Ratatui (UI rendering)
* Crossterm (terminal control and input)
* notify-rust (notifications)
* Serde (settings persistence)

---

## Project Structure

```text
src/
├── main.rs      # app loop and input handling
├── app.rs       # state and core logic
├── ui.rs        # rendering
├── notify.rs    # notifications
├── config.rs    # settings persistence
```

---

## Future Improvements

* Config directory support (`~/.config/...`)
* Custom sound selection
* Theme system
* Session history and analytics
* Cross-platform notification improvements

---

## License

MIT

---

Built by [swayam-rajput](https://github.com/swayam-rajput)
