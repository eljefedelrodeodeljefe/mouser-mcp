# Order from Claude Code with a human in the loop: the model prepares the order, the person
# confirms it in a form the model cannot see or answer (MCP elicitation).
# Render from the repository root:  vhs demo/order.tape
# Uses the demo Mouser API (demo/fake_mouser.py): the "order" gets a DEMO- number, nothing is bought.
#
# The form keys below assume Claude Code's form dialog: Down/Up move within the list, Tab
# moves to the next field, Space toggles, Enter submits. If a Claude Code release changes them,
# adjust the block marked FORM.

Output demo/order.gif
Output demo/order.mp4

Require claude
Require python3

Set Shell bash
Set FontSize 18
Set Width 1400
Set Height 900
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 35ms
Set WindowBar Colorful

Hide
Type "demo/setup.sh && clear" Enter
Sleep 8s
Show

Type `claude --strict-mcp-config --mcp-config demo/mcp.json --allowedTools "mcp__mouser__*"`
Sleep 500ms
Enter
Sleep 5s

Type "Put the in-stock parts into a new Mouser cart: 2x ZX10R-2-183-S+, 2x FMAT7467-6, 1x 415-0033-M1.0, 1x 415-0033-M2.0, 2x ST0645. Then order it: UPS Express Saver, PayPal."
Sleep 800ms
Enter
# cart, order options, preview, then submit_order opens the form
Sleep 45s

# FORM: "Order lines" (all selected) first, then "Place this order" (unticked).
# Show the list, then move to the checkbox, tick it, submit.
Sleep 3s
Down
Sleep 700ms
Down
Sleep 700ms
Down
Sleep 700ms
Down
Sleep 700ms
Down
Sleep 700ms
Tab
Sleep 1s
Space
Sleep 1.5s
Enter
Sleep 12s

Sleep 4s
Ctrl+C
Ctrl+C
