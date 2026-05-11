# pomodoro-tui

A terminal Pomodoro timer with a ludicrous animated progress bar.
Built with Rust + ratatui + crossterm.

```
  FOCUS                     ● ● ○ ○

        24:07

  ┌─────────────────────────────────── 42% ┐
  │ ████▓▒░▓██▓░▒▓██░▒▓█▓▒░▒▓█░▒░░░░░░░░ │
  └────────────────────────────────────────┘

        stay focused

  [space] pause/resume   [s] skip   [q] quit
```

---

## Build and run

```bash
git clone <this repo>
cd pomodoro-tui
cargo run --release
```

Requires Rust stable (1.74+). Install at https://rustup.rs if needed.

---

## Keybinds

| Key     | Action                        |
|---------|-------------------------------|
| `space` | Pause / resume                |
| `s`     | Skip current phase            |
| `enter` | Advance after phase completes |
| `q`     | Quit                          |

---

## Architecture

```
src/
  main.rs   -- Terminal setup/teardown, event loop
  app.rs    -- State machine (timer logic, phase transitions)
  ui.rs     -- All rendering (ratatui widgets + chaos bar)
```

### The three-layer pattern

Every well-structured ratatui app separates these concerns:

1. **State** (`app.rs`) -- A plain Rust struct with no knowledge of terminals.
   Knows what time it is, what phase we're in, whether we're paused.
   Pure logic, easy to test.

2. **Renderer** (`ui.rs`) -- Takes a `&App` (read-only!) and paints a frame.
   Never mutates state. Think of it as a pure function: `App -> Screen`.

3. **Event loop** (`main.rs`) -- The glue. Calls `terminal.draw(render)`,
   polls for input, calls mutating methods on App, repeats.

### How the chaos bar works

The bar is a `Vec<Span>` built fresh every frame in `chaos_bar_spans()`.

A `Span` in ratatui is a piece of styled text. Chaining them into a `Line`
and rendering via `Paragraph` gives us per-character color and style control,
which is how we get the animated flowing effect.

Three zones:

```
[ filled ████▓▒░ | shimmer ▓▒░ | empty ░░░░░░░░ ]
          0..filled          filled..w
```

- **Filled**: cycles through Unicode block chars per tick, each column at
  a different animation phase so it looks like flowing lava
- **Shimmer**: bright braille characters right at the leading edge, cycling
  faster than the fill for a "spark" effect
- **Empty**: static light-shade `░` to show the remaining distance

Color shifts from green -> cyan -> blue as progress rises, then orange
at 85%, then flickering red/yellow in the final 5%.

### The event loop explained

```
loop {
    terminal.draw(|f| render(f, &app));   // render current state

    let timeout = time_until_next_tick();
    if event::poll(timeout)? {            // wait for input, max timeout
        handle_key(event::read()?);       // handle it immediately
    }

    if due_for_tick() {                   // ~10fps
        app.tick();                       // advance timer state
    }
}
```

`event::poll(timeout)` is the key insight: it blocks for AT MOST `timeout`
duration. If no key is pressed, we fall through and tick anyway. If a key
arrives early, we handle it without waiting. This gives us responsive input
AND a stable tick rate with a single thread and no async runtime.

### Why ratatui and not a raw ANSI string mess

ratatui gives you:
- A layout system (split areas into columns/rows with constraints)
- Widgets (Block, Paragraph, Gauge, Table, List, etc.)
- Frame diffing (only rewrites changed cells, no flicker)
- Works on Windows, macOS, Linux

crossterm gives you:
- Raw mode, alternate screen buffer
- Cross-platform key event reading
- Cursor hide/show

Together they cover everything a TUI app needs without reimplementing
terminal control codes by hand.

---

## Customizing durations

In `src/app.rs`, near the top:

```rust
pub const WORK_DURATION: Duration = Duration::from_secs(25 * 60);
pub const SHORT_BREAK:   Duration = Duration::from_secs(5 * 60);
pub const LONG_BREAK:    Duration = Duration::from_secs(15 * 60);
pub const LONG_BREAK_AFTER: u32 = 4;
```

Change these and recompile. That's it.

## Customizing the bar chaos

In `src/ui.rs`, `FILL_CHARS` and `BRAILLE` control what characters fill the bar.
Change the timing divisors in `chaos_bar_spans()` (`tick / 2`, `tick * 3`)
to make the animation faster or slower per zone.

The `progress_color()` function controls the gradient -- it's just a match
on thresholds, easy to extend.
