import md0

MARKDOWN: str = """\
ALWAYS THE END
---
Song by JHung

Could I take back all the things I've said?

Everything that you've seen me do

Every moment we've spent together

Everything that we've been through

Maybe I wanted to think you cared

Wanted to think it could be true

It's always the end when I see things different

Always the end when I see you

Please!

Don't you forget about me, no

'Cause I can't let this go

Don't forget to let me know

Please!

Don't you forget about me, no

'Cause I can't let this go

Don't forget to let me know
"""

print(md0.tokens_to_html(md0.parse(MARKDOWN)))
