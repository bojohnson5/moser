use rodio::buffer::SamplesBuffer;

pub fn sine_wave_samples(freq: f32, duration_sec: f32, sample_rate: usize) -> Vec<f32> {
    let sample_count = (duration_sec * sample_rate as f32) as usize;
    (0..sample_count)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            0.9 * (2.0 * std::f32::consts::PI * freq * t).sin()
        })
        .collect()
}

pub struct MorseAudio {
    pub dit: Vec<f32>,
    pub dah: Vec<f32>,
    pub gap1: Vec<f32>,
    pub gap3: Vec<f32>,
    pub gap7: Vec<f32>,
    pub sample_rate: usize,
}

impl MorseAudio {
    pub fn new(wpm: u32, tone_freq: f32, sample_rate: usize) -> Self {
        let dit_len = 1200.0 / wpm as f32 / 1000.0;
        let dah_len = 3.0 * dit_len;
        let gap1_len = dit_len;
        let gap3_len = 3.0 * dit_len;
        let gap7_len = 7.0 * dit_len;

        Self {
            dit: sine_wave_samples(tone_freq, dit_len, sample_rate),
            dah: sine_wave_samples(tone_freq, dah_len, sample_rate),
            gap1: vec![0.0; (gap1_len * sample_rate as f32) as usize],
            gap3: vec![0.0; (gap3_len * sample_rate as f32) as usize],
            gap7: vec![0.0; (gap7_len * sample_rate as f32) as usize],
            sample_rate,
        }
    }

    pub fn morse_to_audio(&self, morse_str: &str) -> Vec<f32> {
        let mut out = Vec::new();
        if morse_str == " " {
            out.extend(&self.gap7);
        } else {
            for sym in morse_str.chars() {
                match sym {
                    '.' => out.extend(&self.dit),
                    '-' => out.extend(&self.dah),
                    _ => {}
                }
                out.extend(&self.gap1);
            }
            if out.len() >= self.gap1.len() {
                out.truncate(out.len() - self.gap1.len());
                out.extend(&self.gap3);
            }
        }
        out
    }

    pub fn to_source(&self, samples: Vec<f32>) -> SamplesBuffer {
        SamplesBuffer::new(1, self.sample_rate as u32, samples)
    }
}
