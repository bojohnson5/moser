mod app;
mod audio;
mod lesson;
mod morse;
mod scores;
mod ui;

use app::App;
use clap::Parser;

use std::error::Error;

#[derive(Parser, Debug)]
struct Args {
    /// character speed
    #[arg(short, long, default_value_t = 20)]
    wpm: u32,

    /// effective overall wpm
    #[arg(long, default_value_t = 15)]
    effective_wpm: u32,

    /// tone frequency (Hz)
    #[arg(short, long, default_value_t = 600.0)]
    tone_freq: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut app = App::new(args.wpm, args.effective_wpm, args.tone_freq)?;
    app.run()?;

    Ok(())
}
