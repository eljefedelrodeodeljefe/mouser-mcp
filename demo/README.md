# Demo recordings

Terminal recordings of `mouser-mcp` in Claude Code, scripted with [VHS](https://github.com/charmbracelet/vhs)
so they can be re-rendered after any change. They run against `fake_mouser.py`, a stand-in for
the Mouser API with fixed demo data: no account, no key, and the order scene buys nothing (the
order gets a `DEMO-…` number).

| Tape | Shows |
|---|---|
| `price-bom.tape` | A BOM priced at Mouser from one prompt: stock, price breaks, a short line flagged |
| `order.tape` | Cart, preview and order, with the purchase confirmed by the person in a form the model cannot answer |

## Render

Needs `claude` (logged in), `python3`, `cargo`, and VHS with its dependencies `ttyd` and
`ffmpeg`:

```sh
go install github.com/charmbracelet/vhs@latest   # or a release binary
# ttyd: https://github.com/tsl0922/ttyd/releases
vhs demo/price-bom.tape
vhs demo/order.tape
```

Run from the repository root. Each tape builds the server, starts the demo API hidden before
the first frame, and writes a GIF and an MP4 next to itself. The model's answers vary from run
to run; render again if a take is not right. `order.tape` drives the confirmation form with
the arrow keys; see the comment at its top if the form's keys change.

## Try it by hand

```sh
cargo build --release -p mouser-mcp
python3 demo/fake_mouser.py &
claude --strict-mcp-config --mcp-config demo/mcp.json
```
