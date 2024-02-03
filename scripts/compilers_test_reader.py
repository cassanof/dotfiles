import argparse
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument("dir", help="Directory to search for test files")
parser.add_argument(
    "ext", help="File extension to search for (e.g. 'adder' or 'boa')")
args = parser.parse_args()

files = list(Path(args.dir).rglob(f"*.{args.ext}"))
for file in files:
    print(f"## File path: {file}")
    contents = file.read_text()
    print(f"### Code:\n```\n{contents}\n```")
    expected_output = file.with_suffix(".out")
    if expected_output.exists():
        print(f"### Expected output:\n```\n{expected_output.read_text()}\n```")
