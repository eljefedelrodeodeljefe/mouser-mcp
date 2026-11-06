# Price a BOM at Mouser from Claude Code: live stock, price breaks, shortages.
# Render from the repository root:  vhs demo/price-bom.tape
# Uses the demo Mouser API (demo/fake_mouser.py): fixed data, no account needed.

Output demo/price-bom.gif
Output demo/price-bom.mp4

Require claude
Require python3

Set Shell bash
Set FontSize 18
Set Width 1400
Set Height 820
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

Type "Price this BOM at Mouser and flag anything I can't get: 2x ZX10R-2-183-S+, 2x FMAT7467-6, 1x 415-0033-M1.0, 1x 415-0033-M2.0, 2x ST0645, 1x ZFRSC-42-S+"
Sleep 800ms
Enter
Sleep 30s

Sleep 4s
Ctrl+C
Ctrl+C
