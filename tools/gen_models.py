#!/usr/bin/env python3
"""Generate crates/mouser/src/models.rs from Mouser's Swagger 2.0 documents.

    curl -sS -o /tmp/mouser-v1.json https://api.mouser.com/api/docs/V1
    curl -sS -o /tmp/mouser-v2.json https://api.mouser.com/api/docs/V2
    python3 tools/gen_models.py /tmp/mouser-v1.json /tmp/mouser-v2.json > crates/mouser/src/models.rs
    cargo fmt

Every definition becomes a struct with Mouser's own field names (serde renames) and the
Swagger descriptions as doc comments. Request types keep their required fields as plain
values; everything else is an Option, because Mouser omits fields freely in responses.
Fields the documents do not describe still arrive through `extra`, so nothing Mouser sends is
dropped.
"""

import json
import keyword
import re
import sys

# Definitions sent to Mouser: required fields stay required. Everything else is a response.
REQUEST = re.compile(r"(Request|RequestRoot|RequestType|RequestRootType)$|^OrderAddress$")
RUST_KEYWORDS = set(keyword.kwlist) | {"type", "match", "ref", "self", "mod", "use", "fn", "impl", "loop", "move"}


def snake(name: str) -> str:
    s = name.replace("-", "_")
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)
    s = s.lower()
    return f"r#{s}" if s in RUST_KEYWORDS else s


def rust_type(p: dict) -> str:
    if "$ref" in p:
        return p["$ref"].split("/")[-1]
    t = p.get("type")
    if t == "array":
        return f"Vec<{rust_type(p['items'])}>"
    return {"string": "String", "integer": "i64", "number": "f64", "boolean": "bool"}.get(t, "serde_json::Value")


def doc(text: str, indent: str) -> list[str]:
    text = re.sub(r"\s+", " ", text or "").strip()
    if not text:
        return []
    # wrap at ~96 columns
    words, lines, cur = text.split(" "), [], ""
    for w in words:
        if len(cur) + len(w) + 1 > 92:
            lines.append(cur)
            cur = w
        else:
            cur = f"{cur} {w}".strip()
    lines.append(cur)
    return [f"{indent}/// {ln.replace('[', '\\[').replace(']', '\\]')}" for ln in lines]


def main() -> int:
    defs: dict = {}
    for path in sys.argv[1:]:
        defs.update(json.load(open(path))["definitions"])
    out = [
        "//! Mouser API data types, generated from Mouser's Swagger documents",
        "//! (<https://api.mouser.com/api/docs/V1>, <https://api.mouser.com/api/docs/V2>) by",
        "//! `tools/gen_models.py`. Do not edit by hand: regenerate.",
        "",
        "use serde::{Deserialize, Serialize};",
        "",
    ]
    for name in sorted(defs):
        d = defs[name]
        req = bool(REQUEST.search(name))
        required = set(d.get("required", [])) if req else set()
        out += doc(d.get("description", ""), "")
        out.append("#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]")
        out.append('#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]')
        out.append(f"pub struct {name} {{")
        for field, p in d.get("properties", {}).items():
            ty = rust_type(p)
            desc = p.get("description", "")
            if "enum" in p:
                desc = (desc + " " if desc else "") + "One of: " + ", ".join(p["enum"]) + "."
            out += doc(desc, "    ")
            attrs = f'rename = "{field}"'
            if field not in required:
                attrs += ', default, skip_serializing_if = "Option::is_none"'
                # Swagger documents enums as strings; the API sends their numbers.
                if "enum" in p and ty == "String":
                    attrs += ', deserialize_with = "crate::lenient::enum_text"'
            out.append(f"    #[serde({attrs})]")
            out.append(f"    pub {snake(field)}: {ty if field in required else f'Option<{ty}>'},")
        if not req:
            out.append("    /// Fields Mouser sent that the Swagger documents do not describe.")
            out.append("    #[serde(flatten)]")
            out.append("    pub extra: serde_json::Map<String, serde_json::Value>,")
        out.append("}")
        out.append("")
    sys.stdout.write("\n".join(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
