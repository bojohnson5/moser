mod audio;
mod lesson;
mod morse;
mod scores;

use rodio::Sink;
use scores::ScoreData;
use std::{
    cmp::min,
    io::{self, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scores: ScoreData = confy::load("morse", None)?;
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

    let mut correct_chars = 0;
    for i in 0..min(user_input.len(), morse_text.len()) {
        if user_input.get(i..i + 1) == morse_text.get(i..i + 1) {
            correct_chars += 1;
        }
    }
    let accuracy = (correct_chars * 100 / morse_text.len()) as u32;
    scores
        .lessons
        .entry(current_lesson as u8)
        .or_default()
        .push(accuracy);
    confy::store("moser", None, &scores)?;

    sink.sleep_until_end();
    Ok(())
}
