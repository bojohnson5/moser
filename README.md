# Moser

**Moser** is a terminal-based Morse code training program written in Rust.  
It helps you learn Morse code using the **Koch method** (gradually introducing characters) combined with **Farnsworth spacing** for effective practice.  
Moser provides real-time audio playback of Morse code and a text-based user interface (TUI) powered by the Rust `ratatui` library. This allows you to practice copying Morse code at high character speed with longer gaps, improving your listening skills and accuracy.

---

## ✨ Features

- **Koch Method Lessons:** Start with two characters (K and M) and gradually add one new character per lesson following the standard LCWO Koch sequence. This approach ensures you always practice at full speed for known characters while steadily expanding your repertoire.  
- **Farnsworth Spacing:** Separate controls for character speed vs. overall effective speed. You can train with characters at (for example) 20 WPM while spacing them out to 15 WPM overall, making it easier to learn the rhythm of each Morse character without getting overwhelmed.  
- **Terminal User Interface (TUI):** Moser runs in your terminal with an interactive interface built using `ratatui`. The TUI includes:  
  - A scrollable lesson list showing all lessons and the new character(s) introduced in each.  
  - Lesson details for the currently selected lesson (lesson number, new characters, current WPM settings).  
  - A progress chart of your last 10 scores, with a 90% accuracy threshold line to track your improvement.  
  - An input pane where your typed responses appear in real time as you copy the Morse code.  
  - A pop-up preview window for practicing new letters in isolation (triggered by the preview key, see below).  
- **Audio Playback:** Moser generates Morse code audio in real time using the `rodio` audio library. You’ll hear each dot and dash at the specified tone (default 600 Hz) and speed. This real-time playback lets you train your listening and typing reflexes as if you were copying live Morse.  
- **Scoring & Persistence:** After each lesson attempt, Moser calculates your accuracy using Levenshtein distance (so near-misses still count partially). Your score (percentage of correct characters) is recorded per lesson in a local TOML config file via `confy`. Scores are saved automatically between sessions. When you achieve 90% or higher accuracy on a lesson, Moser will suggest that you move on to the next lesson (the progress chart’s 90% line helps visualize this).  
- **Letter Preview Mode:** If you want to focus on new characters before attempting a full lesson, Moser offers a preview mode. Pressing `p` lets you hear the new letter(s) for the selected lesson on repeat (with proper Morse timing) in a pop-up window. This helps you familiarize yourself with the sound of new Morse characters in isolation. You can exit the preview and return to the menu at any time (see key bindings below).

---

## 🚀 Installation

### Prerequisites
- **Rust** – You’ll need a Rust installation (latest stable toolchain) to build Moser from source. You can get Rust from the official website or via `rustup`.  
- **Terminal with Unicode/Color Support** – Moser’s TUI uses Unicode characters and colors for the interface, so use a modern terminal that supports these features.  
- **Audio Output** – Ensure your system can play sound (speakers or headphones) as Moser will output Morse code audio tones.  

### Building from Source
You can clone the repository and build the project with Cargo:

```bash
git clone https://github.com/bojohnson5/moser.git
cd moser
cargo build --release
```

This will produce a binary in the `target/release` directory. (You can also use `cargo run --release` to compile and run it in one step.)

> **Note:** Moser is not yet available via crates.io, so building from source is the primary installation method.

---

## 📖 Usage

Once you have compiled the project, you can run the **Moser** binary to start the Morse trainer. By default, Moser uses a character speed of **20 WPM** and an effective speed of **15 WPM** (Farnsworth timing), with a **600 Hz** tone frequency. You can customize these settings using command-line options:

- `-w, --wpm <N>` – Set the Morse character speed in WPM (letters are sent at this speed). Default is 20 WPM.  
- `--effective-wpm <N>` – Set the effective overall speed in WPM (controls extra spacing between characters). Default is 15 WPM.  
- `-t, --tone-freq <Hz>` – Set the tone frequency for the Morse audio in hertz. Default is 600.0 Hz.  

For example, to run Moser at 25 WPM characters, 20 WPM effective speed, and a 700 Hz tone, use:

```bash
cargo run --release -- -w 25 --effective-wpm 20 -t 700
```

### Workflow

1. **Select a Lesson:** Upon start, you’ll see a table of lessons. Use the **Up/Down arrow keys** (or **`k`/`j` vi-keys) to move the selection up or down. Each lesson is numbered and shows which new character(s) it introduces. The first lesson starts with **K** and **M**, and each subsequent lesson adds one new character. Press **Enter** to select the highlighted lesson and begin that practice session.  
   - *(Optional)* **Preview the new letters:** Before pressing Enter, you can press **`p`** to hear the new character(s) for the selected lesson in a loop. This opens a “Letter Practice” popup where the new Morse letters repeat at the set speed, helping you get used to them. Press **Esc** to close the preview and return to the lesson list.  
2. **Listen and Type:** Once you start a lesson, Moser will begin playing a series of Morse code characters (random groups of letters, 5 characters per group) for that lesson. Listen to the Morse audio and **type the corresponding letters** on your keyboard as you hear them. The characters you type will appear in the **Your Input** box in the interface. You can use **Backspace** to correct any mistakes while typing. (If you need to pause or give up on the current lesson, press **Esc** to stop the audio and return to the lesson picker.)  
3. **Submit and Score:** After the Morse sequence finishes (or whenever you are done typing), press **Enter** to submit your attempt. Moser will then compare your input to the expected text and calculate your accuracy. In the interface, your input will be shown with correct characters highlighted in **green** and any errors highlighted in **red**. You’ll also see an accuracy percentage for that attempt. Moser uses Levenshtein distance to score your input, which means it accounts for insertions or deletions – helping give a fair accuracy score even if your input is slightly misaligned. If your accuracy is **90% or above**, Moser considers you ready to move to the next character; a success at this level is a good indicator to proceed to the next lesson (the progress chart on the right highlights the 90% threshold with a line for reference).  
4. **Progress and Repeat:** Close the results (if a popup is shown) with **Esc**, which returns you to the lesson selection. You can now repeat the same lesson for additional practice or use the arrow keys to select the next lesson. All your scores are saved automatically to a config file, so you can track your progress over time. When you revisit Moser, the chart will display your last 10 scores for each lesson, allowing you to monitor improvements. Continue through the lessons at your own pace until you’ve learned the entire Morse code alphabet (letters, numbers, and punctuation).  
5. **Quit:** You can exit Moser at any time by pressing **`q`**, which will quit the application. Your progress is preserved, so you can always come back later and resume training from where you left off.

---

## 🔑 Key Bindings

For quick reference, here are the key controls available in Moser:

### Global
- `q` – Quit the application (exit Moser)  

### Lesson Picker (Main Menu)
- `↑` / `k` – Move selection up (previous lesson)  
- `↓` / `j` – Move selection down (next lesson)  
- `Enter` – Start the selected lesson (begin audio playback and switch to typing mode)  
- `p` – Preview new letters of selected lesson (open letter practice popup)  

### Typing Mode (During Lesson)
- *(any letter key)* – Type the letter you hear (adds it to your input)  
- `Backspace` – Delete the last character (correct mistakes)  
- `Enter` – Submit your input for scoring (ends the lesson and returns to menu)  
- `Esc` – Cancel the lesson and return to the lesson picker (stop audio playback)  

### Letter Practice Mode (Preview Popup)
- `Esc` – Close the preview window and return to the lesson picker  

---
