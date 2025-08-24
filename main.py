import simpleaudio as sa
import numpy as np
import random
from moser.lessons import KOCH_CHAR_SEQUENCE
from moser.morse_map import morse


class Morse:
    @staticmethod
    def generate_wave(tone, length, sample_rate):
        t = np.linspace(0.0, length, int(length * sample_rate))
        note = np.sin(2 * np.pi * tone * t)
        note *= 32767 / np.max(
            np.abs(note)
        )  # ensures the samples are within 16-bit range

        return note.astype(np.int16)

    def __init__(self, wpm=20, tone=600, sample_rate=8000):
        dit = (
            1200 / wpm / 1000
        )  # this is the standard method to determine dit length in sec
        dah = dit * 3
        self.dit_note = self.generate_wave(tone, dit, sample_rate)
        self.dah_note = self.generate_wave(tone, dah, sample_rate)
        self.silent1_note = np.zeros(int(dit * sample_rate), dtype=np.int16)
        self.silent3_note = np.zeros(int(dit * 3 * sample_rate), dtype=np.int16)
        self.silent7_note = np.zeros(int(dit * 7 * sample_rate), dtype=np.int16)

        self.audio_sequence = []
        self.sample_rate = sample_rate
        self.current_lesson = 2
        self.word_sequence = self.lesson_sequence()
        self.morse_sequence = self.convert_words_to_morse()
        self.convert_morse_to_audio()

    def play_audio(self):
        sequence = np.concatenate(self.audio_sequence)
        play_sequence = sa.play_buffer(sequence, 1, 2, self.sample_rate)
        play_sequence.wait_done()

    def lesson_sequence(self):
        letters = KOCH_CHAR_SEQUENCE[: self.current_lesson]
        words = ["".join(random.choice(letters) for _ in range(5)) for _ in range(10)]
        words = " ".join(words)

        return words

    def convert_words_to_morse(self):
        morse_letters = [morse[key] for key in self.word_sequence]

        return morse_letters

    def convert_morse_to_audio(self):
        for val in self.morse_sequence:
            if val == " ":
                self.audio_sequence.append(self.silent7_note)
                continue

            for sym in val:
                if sym == ".":
                    self.audio_sequence.append(self.dit_note)
                elif sym == "-":
                    self.audio_sequence.append(self.dah_note)
                self.audio_sequence.append(self.silent1_note)

            self.audio_sequence[-1] = self.silent3_note


m = Morse()
m.play_audio()
