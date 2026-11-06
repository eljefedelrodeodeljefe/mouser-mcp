#!/usr/bin/env python3
"""Cuts the README's two clips out of demo/story.mp4 (rendered by demo/story.tape).

Most of a take is the screen standing still while the model works. This finds those still
stretches (ffmpeg's freezedetect), shortens each to `--wait` seconds, except the ones named
with `--hold START:SECONDS` (an answer to read, the form), and writes two GIFs, split at
`--split` (a time in the take, the start of scene 3):

    demo/search.gif   scenes 1-2: the backordered BOM, then in-stock alternatives
    demo/order.gif    scenes 3-4: cart, preview and the confirmation form

Every take times differently: read the times off the MP4 (or `--list`) and pass them.

    demo/trim.py --list
    demo/trim.py --split 120.8 --hold 25.4:7 --hold 82.5:10 --hold 139.5:5 --hold 170.3:4 --hold 206.4:9

Needs ffmpeg and ffprobe. Standard library only.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys

SRC = "demo/story.mp4"
FILTER = "fps=6,split[a][b];[a]palettegen=max_colors=48:stats_mode=diff[p];[b][p]paletteuse=dither=none:diff_mode=rectangle"


def duration() -> float:
    out = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", SRC],
        check=True, capture_output=True, text=True,
    ).stdout
    return float(out)


def freezes(total: float) -> list[tuple[float, float]]:
    """Still stretches of at least 2.5 s, as (start, end)."""
    log = subprocess.run(
        ["ffmpeg", "-v", "info", "-i", SRC, "-vf", "freezedetect=n=0.002:d=2.5", "-map", "0:v", "-f", "null", "-"],
        check=True, capture_output=True, text=True,
    ).stderr
    starts = [float(x) for x in re.findall(r"freeze_start: ([0-9.]+)", log)]
    ends = [float(x) for x in re.findall(r"freeze_end: ([0-9.]+)", log)]
    ends += [total] * (len(starts) - len(ends))  # a freeze running to the end has no end line
    return list(zip(starts, ends))


def keep(total: float, still: list[tuple[float, float]], wait: float, holds: dict[float, float]) -> list[tuple[float, float]]:
    """The intervals to keep: everything but the tail of each still stretch."""
    out, t = [], 0.0
    for start, end in still:
        hold = next((s for h, s in holds.items() if abs(h - start) < 0.6), wait)
        cut_from = min(end, start + hold)
        if cut_from > t:
            out.append((t, cut_from))
        t = max(t, end)
    if t < total:
        out.append((t, total))
    return out


def gif(intervals: list[tuple[float, float]], lo: float, hi: float, path: str) -> None:
    parts = [(max(a, lo), min(b, hi)) for a, b in intervals if b > lo and a < hi]
    expr = "+".join(f"between(t,{a:.2f},{b:.2f})" for a, b in parts)
    vf = f"select='{expr}',setpts=N/FRAME_RATE/TB,{FILTER}"
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", SRC, "-vf", vf, path], check=True)
    length = sum(b - a for a, b in parts)
    print(f"{path}: {length:.0f} s from {hi - lo:.0f} s of the take")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--list", action="store_true", help="print the still stretches and stop")
    ap.add_argument("--split", type=float, help="time in the take where scene 3 starts")
    ap.add_argument("--hold", action="append", default=[], metavar="START:SECONDS",
                    help="keep this still stretch (starting near START) for SECONDS")
    ap.add_argument("--wait", type=float, default=2.0, help="seconds kept of any other still stretch")
    a = ap.parse_args()
    total = duration()
    still = freezes(total)
    if a.list or a.split is None:
        for s, e in still:
            print(f"{s:7.1f} - {e:7.1f}  ({e - s:4.1f} s still)")
        return 0 if a.list else 2
    holds = {float(h.split(":")[0]): float(h.split(":")[1]) for h in a.hold}
    kept = keep(total, still, a.wait, holds)
    gif(kept, 0.0, a.split, "demo/search.gif")
    gif(kept, a.split, total, "demo/order.gif")
    return 0


if __name__ == "__main__":
    sys.exit(main())
