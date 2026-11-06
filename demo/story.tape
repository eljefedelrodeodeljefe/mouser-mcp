# One Claude Code session against the demo Mouser API (demo/fake_mouser.py), following a real
# session from 2026-11-06: a BOM for a 2-3 GHz bench test is mostly backordered, the agent
# finds in-stock alternatives, carts them and orders, and the person confirms the order in a
# form the model cannot answer. Nothing is bought: the order gets a DEMO- number.
#
# Render from the repository root:  vhs demo/story.tape
# then cut the two README clips:    demo/cut.sh
#
# Claude Code runs in an empty folder (~/.cache/mouser-mcp-demo/bench, made by demo/setup.sh).
# The first time, Claude Code asks whether to trust it: run `claude` there once and accept.
#
# The waits are fixed: the model's answers take a different time on every run. If a scene is
# cut short, raise its Sleep. The form keys are Claude Code 2.1's (checked by hand): right
# expands the list, down walks it and moves on to the checkbox, space ticks, down to Accept,
# enter submits.

Output demo/story.mp4

Require claude
Require python3

Set Shell bash
Set FontSize 16
Set Width 1400
Set Height 1000
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms
Set WindowBar Colorful

Hide
# A recording started from inside Claude Code inherits its session markers: clear them.
Type "unset $(compgen -v CLAUDE); demo/setup.sh && cd ~/.cache/mouser-mcp-demo/bench && clear" Enter
Sleep 8s
Type `claude --strict-mcp-config --mcp-config mcp.json --allowedTools "mcp__mouser__*" --append-system-prompt "Before each Mouser tool call, say in one short line which tool you call and why."`
Enter
Sleep 6s
Show

# Scene 1: the first-choice BOM, priced
Type "Price this BOM at Mouser and flag what cannot ship soon: 2x ZFRSC-42-S+, 2x BW-S6W2+, 1x CBL-1M-SMSM+, 1x CBL-2M-SMSM+, 2x CBL-1FT-SMSM+, 2x ANNE-50+"
Sleep 600ms
Enter
Sleep 26s

# Scene 2: in-stock alternatives that cover 2-3 GHz
Type "Are there no alternative parts? This is for a 2 to 3 GHz bench and I need the parts this week."
Sleep 600ms
Enter
Sleep 60s

# Scene 3: the cart
Type "Put the in-stock set in a new cart: the ZX10R splitters, the Fairview pads and terminations, and the Cinch 1 m and 2 m cables. Skip the 1 ft jumpers, my bench cables reach."
Sleep 600ms
Enter
Sleep 20s

# Scene 4: the order, confirmed by the person in the form
Type "Order it with UPS Express Saver and PayPal."
Sleep 600ms
Enter
Sleep 38s

Sleep 3s
Right
Sleep 2s
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 1s
Space
Sleep 1.5s
Down
Sleep 1s
Enter
Sleep 15s

Sleep 3s
