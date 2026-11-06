# Demo

A recording of `mouser-mcp` in Claude Code, scripted with [VHS](https://github.com/charmbracelet/vhs)
so it can be rendered again after any change. It runs against `fake_mouser.py`, a stand-in for
the Mouser API with the parts, prices, stock and lead times Mouser showed on 2026-11-06. It
needs no account and no key, and the order buys nothing: it goes to an invented address and
gets a `DEMO-…` number.

`story.tape` is one Claude Code session in four scenes, following a real session:

1. A BOM for a 2–3 GHz bench test is priced: €732.26, five of six lines backordered.
2. "Are there no alternative parts?": in-stock alternatives that cover 2–3 GHz, €493.78.
3. The in-stock set goes into a new cart.
4. The order: preview, then the confirmation form, which the person accepts.

`trim.py` cuts it into the README's two clips, `search.gif` (scenes 1–2) and `order.gif`
(scenes 3–4), shortening the stretches where the screen stands still while the model works.

## Render

Needs `claude` (logged in), `python3`, `cargo`, `ffmpeg`, and VHS with `ttyd`:

```sh
go install github.com/charmbracelet/vhs@latest   # or a release binary
# ttyd: https://github.com/tsl0922/ttyd/releases
vhs demo/story.tape                 # writes demo/story.mp4 (not committed)
demo/trim.py --list                 # the still stretches, with their times
demo/trim.py --split 120.8 --hold 25.4:7 --hold 82.5:10 --hold 139.5:5 --hold 170.3:4 --hold 206.4:9
```

Run from the repository root. The tape builds the server, starts the stand-in hidden before
the first frame, and runs `claude` in a bench folder, `~/.cache/mouser-mcp-demo/bench`, that
holds only the session's configuration, so the session shows a user's bench and this
repository stays out of it. The first time, Claude Code asks
whether to trust that folder: run `claude` there once and accept.

The model's answers vary from take to take. Check the frames before committing (a scene can
end differently, for example on the order ceiling), and read the times for `trim.py` off the
new take: `--split` is where scene 3's prompt is typed, and each `--hold` is an answer to keep
on screen. The form keys in the tape assume five order lines.

## Try it by hand

```sh
cargo build --release -p mouser-mcp
demo/setup.sh                       # starts the stand-in and prepares the bench folder
cd ~/.cache/mouser-mcp-demo/bench && claude
```

The bench folder holds the session's configuration, so `claude` needs no flags: `.mcp.json`
(the server, pointed at the stand-in), `.claude/settings.local.json` (the server approved and
its tools allowed) and `CLAUDE.md` (asks the agent to name each tool it calls).
