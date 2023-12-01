import re

def main():
    with open("input.txt") as f:
        data = f.read()

    lines = data.split("\n")

    p = re.compile(r"(?=([0-9]|one|two|three|four|five|six|seven|eight|nine))")

    output = 0
    for line in lines:
        matches = p.findall(line)
        first, last = matches[0], matches[-1]

        output += convert(first) * 10 + convert(last)

    print(output)

def convert(s):
    if s == "one":
        return 1
    elif s == "two":
        return 2
    elif s == "three":
        return 3
    elif s == "four":
        return 4
    elif s == "five":
        return 5
    elif s == "six":
        return 6
    elif s == "seven":
        return 7
    elif s == "eight":
        return 8
    elif s == "nine":
        return 9
    else:
        return int(s)
        

if __name__ == "__main__":
    main()

