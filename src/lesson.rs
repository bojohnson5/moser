use crate::morse::KOCH_SEQUENCE;
use rand::Rng;

pub fn lesson_text(current_lesson: usize) -> String {
    let letters = &KOCH_SEQUENCE[..current_lesson];
    let mut rng = rand::rng();

    let words: Vec<String> = (0..10)
        .map(|_| {
            (0..5)
                .map(|_| {
                    let idx = rng.random_range(0..letters.len());
                    letters[idx]
                })
                .collect()
        })
        .collect();

    words.join(" ")
}
