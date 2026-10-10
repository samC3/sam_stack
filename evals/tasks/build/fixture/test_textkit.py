import unittest

from textkit import initials, word_count


class WordCountTest(unittest.TestCase):
    def test_counts_words_across_runs_of_whitespace(self):
        self.assertEqual(word_count("a  b\tc"), 3)


class InitialsTest(unittest.TestCase):
    def test_uppercases_first_letters(self):
        self.assertEqual(initials("ada  lovelace"), "AL")


if __name__ == "__main__":
    unittest.main()
