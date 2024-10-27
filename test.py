import md0

MARKDOWN: str = """\
# Docs
[Hello, world!](https://google.com)
"""

print(md0.parse(MARKDOWN))
