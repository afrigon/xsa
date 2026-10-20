import textwrap
from pathlib import Path

from operations.credit import Credit

COLUMNS = 80
BULLET = "- "
BULLET_INDENT = "  "
MIT_LICENSE = """MIT License

Copyright (c) 2026 Alexandre Frigon

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

Third-party content
-------------------

The following content in this pack comes from third parties under their own
terms.
"""


class LicenseWriter:
    def __init__(self, path: Path):
        self.path = path

    def write(self, credits: list[Credit]) -> None:
        sections = [MIT_LICENSE]

        for credit in credits:
            paragraphs = ["\n".join(map(self._wrap, paragraph.split("\n"))) for paragraph in credit.text.split("\n\n")]
            sections.append(credit.heading + "\n\n" + "\n\n".join(paragraphs) + "\n")

        self.path.write_text("\n".join(sections))

    @staticmethod
    def _wrap(line: str) -> str:
        indent = BULLET_INDENT if line.startswith(BULLET) else ""
        return textwrap.fill(line, COLUMNS, subsequent_indent=indent, break_on_hyphens=False)
