# Moser

A terminal-based Morse code trainer written in Rust.  
Moser helps you learn Morse code using the **Koch method** with Farnsworth spacing, real-time audio playback, and a text-based user interface (TUI) powered by [ratatui](https://github.com/tui-rs-revival/ratatui).

---

## ✨ Features

- **Koch Method Lessons**  
  Start with two characters (K and M) and gradually add one new character per lesson following the LCWO sequence.

- **Farnsworth Spacing**  
  Adjust character speed (WPM) and effective overall speed for comfortable practice.

- **Terminal User Interface (TUI)**  
  Built with `ratatui`:
  - Lesson picker (scrollable table of lessons)
  - Lesson details (WPM, effective WPM, new characters)
  - Progress chart (last 10 scores with a 90% threshold line)
  - Input pane for copying practice strings
  - Pop-up window for **letter preview mode**

- **Audio Playback**  
  Real-time Morse audio generated using `rodio`.

- **Scoring & Persistence**  
  User accuracy is calculated with Levenshtein distance and stored per-lesson in a TOML config file via `confy`.  
  - Scores are saved automatically between runs.
  - If accuracy >90%, Moser suggests moving to the next lesson.

- **Letter Preview Mode**  
  Press `p` in the lesson pane to repeatedly hear new letters before practicing.

---

## 🔑 Key Bindings

- **Global**
  - `q` → Quit

- **Lesson Picker**
  - `↑ / k` → Move up
  - `↓ / j` → Move down
  - `Enter` → Start lesson (play audio, switch to typing)
  - `p` → Preview new letters (floating window)

- **Typing Mode**
  - Type as you hear the audio
  - `Backspace` → Delete character
  - `Enter` → Submit, score, and save results
  - `Esc` → Cancel, stop audio, return to picker

- **Letter Practice Mode**
  - `Esc` → Close preview and return to picker

---

## 🚀 Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- A terminal that supports Unicode and colors
- Audio output (for Morse tones)

### Clone and build
```bash
git clone https://github.com/bojohnson5/moser
cd moser
cargo build --release
