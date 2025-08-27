use crate::morse::KOCH_SEQUENCE;
use rand::Rng;

pub fn lesson_text(current_lesson: usize) -> String {
    let count = if current_lesson == 1 {
        2
    } else {
        current_lesson + 1
    };
    let letters = &KOCH_SEQUENCE[..count];
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

pub fn new_letters_for_lesson<'a>(lesson_num: usize) -> &'a [char] {
    if lesson_num == 1 {
        &KOCH_SEQUENCE[0..2]
    } else {
        &KOCH_SEQUENCE[lesson_num..lesson_num + 1]
    }
}
