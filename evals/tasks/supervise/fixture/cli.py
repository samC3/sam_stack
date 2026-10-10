import sys

from dates import parse_date

if __name__ == "__main__":
    print(parse_date(sys.argv[1]).strftime("%A"))
