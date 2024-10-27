# md0

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Somehow correct Markdown parser.

```python
import md0

MARKDOWN: str = """\
# Docs
Hello, everyone! This document is just crazy!
This doesn't become a new paragraph, but rather joined to the previous line with a space.

Crazy, right? You can also use code blocks:
```python
# Fibonacci sequence!
def fib_n(n: int) -> int:
    return 1 if n <= 1 else fib_n(n - 1) + fib_n(n - 2)

print(fib_n(2))
```"""

md0.parse(MARKDOWN)
# [
#     Heading(1, "Docs"),
#     Paragraph(
#         "Hello, everyone! This document is just crazy! This doesn't become a new paragraph, but rather joined to the previous line with a space."
#     ),
#     Paragraph("Crazy, right? You can also use code blocks:"),
#     Code(
#         "python",
#         "# Fibonacci sequence!\ndef fib_n(n: int) -> int:\n    return 1 if n <= 1 else fib_n(n - 1) + fib_n(n - 2)\n\nprint(fib_n(2))\n",
#     ),
# ]
```

Links are attached to `metadata`, so I could save some computation resources lmfao. Don't blame me. PyO3 says it wants `#[derive(Clone)]` which is very inappropriate (but I did it anyways).
