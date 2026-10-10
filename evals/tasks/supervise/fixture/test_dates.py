import unittest
from datetime import date

from dates import parse_date


class ParseDate(unittest.TestCase):
    def test_parses_iso(self):
        self.assertEqual(parse_date("2026-10-08"), date(2026, 10, 8))

    def test_rejects_impossible_day(self):
        with self.assertRaises(ValueError):
            parse_date("2026-02-30")

    def test_rejects_unpadded(self):
        with self.assertRaises(ValueError):
            parse_date("2026-1-5")


if __name__ == "__main__":
    unittest.main()
