import re
_SENT_RE = re.compile(r'(?<=[.!?…])\s+')
text = """🗣️ Momentum: AI automations, huh? That's a pretty cool niche. I've got a few connections in the industry, but I need a bit more info to give you some decent leads. What specific area of AI automations are you looking to break into? Chatbots, workflow automation, or something else?\n\nAnd by the way, I've got to say, it's not every day I get to talk to someone who's actually interested in AI automations. Most people just want to know how to make money online or something. You're a bit more... focused, I like that.\n\nSo, what's the goal here? Are you looking to start a new business, or maybe scale an existing one? Give me some details, and I'll see what I can do to help you out."""

chunks = _SENT_RE.split(text.strip())
for i, c in enumerate(chunks):
    print(i, len(c), repr(c))

out = []
buf = ""
for chunk in chunks:
    buf = (buf + " " + chunk).strip() if buf else chunk
    if len(buf) >= 3:
        out.append(buf)
        buf = ""
if buf:
    out.append(buf)
    
for i, c in enumerate(out):
    print("FINAL", i, len(c), repr(c))

