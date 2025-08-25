mod audio;
mod lesson;
mod morse;

use rodio::Sink;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wpm = 15;
    let tone_freq = 600.0;
    let sample_rate = 44_100;

    let morse_text = lesson::lesson_text(4);

    let map = morse::morse_map();

    let audio = audio::MorseAudio::new(wpm, tone_freq, sample_rate);

    let mut samples: Vec<f32> = Vec::new();
    for ch in morse_text.chars() {
        if let Some(code) = map.get(&ch) {
            samples.extend(audio.morse_to_audio(code));
        }
    }

    let mut stream = rodio::stream::OutputStreamBuilder::open_default_stream()?;
    stream.log_on_drop(false);
    let sink = Sink::connect_new(&stream.mixer());
    let source = audio.to_source(samples);
    sink.append(source);

    println!("Start typing what you hear. Press ENTER when finished:");
    io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;

    println!("\nYou entered: {}", user_input.trim());
    println!("Expected:   {}", morse_text);

    sink.sleep_until_end();
    Ok(())
}
