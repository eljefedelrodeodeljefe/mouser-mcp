#!/usr/bin/env python3
"""A stand-in for the Mouser API with fixed demo data, for recordings and trying the server.

    python3 demo/fake_mouser.py            # listens on 127.0.0.1:8787
    MOUSER_API_BASE=http://127.0.0.1:8787/api mouser-mcp

Nothing here talks to Mouser: searches answer from the parts below, carts live in memory,
and a submitted order gets a demo order number. Prices, stock and lead times are the ones
Mouser showed on 2026-11-06; they are fixed here and do not follow the real catalogue.
"""

import json
import sys
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

# Stock, prices, lead times and on-order dates as Mouser (mouser.de, EUR) showed them on
# 2026-11-06: the first-choice BOM for a 2-3 GHz two-delay-line test, its in-stock
# alternatives, and the out-of-band or costlier options a search also returns.
# (mpn): (mouser number, maker, description, stock, price, unit, lead days, on order)
PARTS = {
    # first choice
    "ZFRSC-42-S+": ("139-ZFRSC-42-S", "Mini-Circuits", "Signal Conditioning 2 Ways Resistive Power Splitter, DC - 4200 MHz, 50 Ohm", "0", "81,38 €", 81.38, 217, [(248, "2026-12-28"), (190, "2027-01-25")]),
    "BW-S6W2+": ("139-BW-S6W2", "Mini-Circuits", "Attenuators - Interconnects FXD ATTEN 2W SMA F/M 6dB", "0", "46,88 €", 46.88, 119, [(270, "2026-11-09"), (510, "2026-11-30")]),
    "CBL-1M-SMSM+": ("139-CBL-1M-SMSM+", "Mini-Circuits", "RF Cable Assemblies Precision Test Cable, 18.0 GHz, 1 m", "0", "106,81 €", 106.81, 119, [(131, "2026-12-07")]),
    "CBL-2M-SMSM+": ("139-CBL-2M-SMSM+", "Mini-Circuits", "RF Cable Assemblies Precision Test Cable, 18.0 GHz, 2 m", "0", "136,99 €", 136.99, 119, [(290, "2027-03-22")]),
    "CBL-1FT-SMSM+": ("139-CBL-1FT-SMSM", "Mini-Circuits", "RF Cable Assemblies Precision Test Cable, 18.0 GHz, 1 ft", "21", "99,41 €", 99.41, 119, [(30, "2026-12-21")]),
    "ANNE-50+": ("139-ANNE-50", "Mini-Circuits", "RF Terminators TERM / SMA-M / 18GHZ", "0", "16,56 €", 16.56, 119, [(1321, "2026-11-16")]),
    # in-stock alternatives
    "ZX10R-2-183-S+": ("139-ZX10R-2-183-S+", "Mini-Circuits", "Signal Conditioning 2 Ways Resistive Power Splitter, DC - 18000 MHz, 50 Ohm", "2", "161,10 €", 161.10, 217, [(90, "2027-06-14")]),
    "FMAT7467-6": ("67-FMAT7467-6", "Fairview Microwave", "Attenuators - Interconnects 6 dB Fixed Attenuator SMA Male to SMA Female DC to 12 GHz, 2 W", "6", "53,10 €", 53.10, 194, []),
    "415-0033-M1.0": ("530-415-0033-M1.0", "Cinch Connectivity Solutions", "RF Cable Assemblies Straight SMA Plug to Straight SMA Plug, 1 m", "402", "17,44 €", 17.44, 124, []),
    "415-0033-M2.0": ("530-415-0033-M2.0", "Cinch Connectivity Solutions", "RF Cable Assemblies Straight SMA Plug to Straight SMA Plug, 2 m", "372", "22,68 €", 22.68, 118, []),
    "ST0645": ("67-ST0645", "Fairview Microwave", "RF Terminators SMA Male Termination 1 W to 6 GHz, VSWR 1.15", "104", "12,63 €", 12.63, 49, []),
    # what a search also finds: out of band, out of stock, or costlier
    "LRPS-2-11+": ("139-LRPS-2-11+", "Mini-Circuits", "Signal Conditioning 2 Ways Core & Wire Power Splitter, 20 - 2000 MHz, 50 Ohm", "95", "34,95 €", 34.95, 119, []),
    "PE2082": ("28-PE2082", "Pasternack", "Signal Conditioning 2 Way SMA Power Divider from DC to 6 GHz Rated at 2 Watts", "10", "412,09 €", 412.09, 16, [(7, "2026-11-24")]),
    "ZX10-2-42-S+": ("139-ZX10-2-42-S+", "Mini-Circuits", "Signal Conditioning 2 Ways DC Pass Power Splitter, 1900 - 4200 MHz, 50 Ohm", "0", "66,57 €", 66.57, 217, [(7, "2026-11-30"), (40, "2026-12-28")]),
    "ATS-1M1F-06DB2W": ("523-ATS-1M1F-06DB2W", "Amphenol RF", "Attenuators - Interconnects SMA Jack to SMA Plug Attenuator, 6 dB, 2W", "70", "55,83 €", 55.83, 98, []),
    "PE6TR1014": ("28-PE6TR1014", "Pasternack", "RF Terminators 1 Watt RF Load Up to 6 GHz with SMA Male", "24", "25,27 €", 25.27, 105, []),
    "SF104/11SMA/11SMA/1000mm": ("455-04SMAMSMAM1000MM", "HUBER+SUHNER", "RF Test Cables MICROWAVE TEST ASSEMBLIES, SMA (male), SMA (male), 50 Ohm, 18 GHz, 1000mm", "212", "179,96 €", 179.96, 70, []),
}
# Words a keyword search treats as the same thing (the demo's prompts use the everyday ones).
SYNONYMS = {
    "splitter": ("splitter", "divider"), "splitters": ("splitter", "divider"), "divider": ("splitter", "divider"),
    "pad": ("attenuator",), "pads": ("attenuator",), "attenuator": ("attenuator",), "attenuators": ("attenuator",),
    "termination": ("termination", "terminator", "load"), "terminations": ("termination", "terminator", "load"),
    "load": ("termination", "terminator", "load"), "cable": ("cable",), "cables": ("cable",),
}
BY_MOUSER = {v[0].lower(): k for k, v in PARTS.items()}
SHIPPING = [(65, "UPS Worldwide Express Saver", 40.0), (21, "DHL International Express", 40.0)]
PAYMENT = [(1081, "PayPal")]
# An invented company, with an address and phone that look like a real order's.
ADDRESS = {
    "AddressLocationTypeID": 2, "FirstName": "Ada", "LastName": "Lovelace", "Company": "Analytical Engines GmbH",
    "AddressOne": "Musterstraße 12", "City": "Berlin", "PostalCode": "10115", "CountryCode": "DE",
    "PhoneNumber": "+49 30 12345678",
}
VAT = 0.19  # German VAT on goods and shipping

CARTS: dict[str, list[dict]] = {}


def part(mpn: str) -> dict:
    mouser, maker, desc, stock, price, _, lead, on_order = PARTS[mpn]
    return {
        "MouserPartNumber": mouser, "ManufacturerPartNumber": mpn, "Manufacturer": maker, "Description": desc,
        "AvailabilityInStock": stock, "Availability": f"{stock} In Stock",
        "AvailabilityOnOrder": [{"Quantity": q, "Date": f"{d}T00:00:00"} for q, d in on_order],
        "LeadTime": f"{lead} Days", "LifecycleStatus": None, "Min": "1", "Mult": "1",
        "PriceBreaks": [{"Quantity": 1, "Price": price, "Currency": "EUR"}],
        "ProductDetailUrl": f"https://www.mouser.de/ProductDetail/{mouser}", "ROHSStatus": "RoHS Compliant",
    }


def keyword_match(mpn: str, keyword: str) -> bool:
    """Every word of the keyword (or one of its synonyms) is in the part number or description."""
    text = f"{mpn} {PARTS[mpn][2]}".lower()
    squeezed = text.replace(" ", "")  # "6dB" and "6 dB", "1m" and "1 m"
    words = [w for w in keyword.lower().replace(",", " ").split() if w]
    return bool(words) and all(
        any(s in text or s in squeezed for s in SYNONYMS.get(w, (w,))) for w in words
    )


def lookup(number: str) -> str | None:
    n = number.strip()
    for mpn in PARTS:
        if mpn.lower() == n.lower():
            return mpn
    return BY_MOUSER.get(n.lower())


def line(item: dict) -> dict:
    mpn = lookup(item["MouserPartNumber"]) or item["MouserPartNumber"]
    mouser, maker, desc, stock, _, unit, _, _ = PARTS.get(mpn, (item["MouserPartNumber"], "", "", "0", "", 0.0, 0, []))
    q = int(item["Quantity"])
    return {
        "MouserPartNumber": mouser, "MfrPartNumber": mpn, "Manufacturer": maker, "Description": desc,
        "Quantity": q, "UnitPrice": unit, "ExtendedPrice": round(unit * q, 2), "MouserATS": int(stock),
        "CartItemCustPartNumber": item.get("CustomerPartNumber") or "", "PackagingChoice": "None", "Errors": [],
    }


def cart_reply(key: str) -> dict:
    lines = [line(i) for i in CARTS.get(key, [])]
    return {"Errors": [], "CartKey": key, "CurrencyCode": "EUR", "CartItems": lines,
            "TotalItemCount": len(lines), "MerchandiseTotal": round(sum(l["ExtendedPrice"] for l in lines), 2)}


def order(body: dict, submit: bool) -> dict:
    o = body.get("Order") or body
    lines = [line(i) for i in CARTS.get(o.get("CartKey", ""), [])]
    code = (o.get("PrimaryShipping") or {}).get("Code") or SHIPPING[0][0]
    method, rate = next(((m, r) for c, m, r in SHIPPING if c == code), (SHIPPING[0][1], SHIPPING[0][2]))
    pay = (o.get("Payment") or {}).get("Method") or PAYMENT[0][0]
    merch = round(sum(l["ExtendedPrice"] for l in lines), 2)
    reply = {
        "Errors": [], "CartGUID": o.get("CartKey"), "CurrencyCode": "EUR", "OrderLines": lines,
        "MerchandiseTotal": merch, "TaxAmount": round((merch + rate) * VAT, 2),
        "OrderTotal": round((merch + rate) * (1 + VAT), 2),
        "ShippingMethod": {"PrimaryCode": code, "PrimaryMethod": method, "PrimaryShippingRate": rate},
        "PaymentMethod": {"Method": pay, "Name": dict(PAYMENT).get(pay, "?")},
        "ShippingAddress": ADDRESS, "BillingAddress": ADDRESS, "SubmitOrder": submit, "OrderID": "0",
    }
    if submit:
        reply["OrderID"] = "DEMO-" + uuid.uuid4().hex[:8].upper()
        CARTS.pop(o.get("CartKey", ""), None)
    return {"Order": reply}


class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        # the path only: the query carries the key, and keys are never logged, even demo ones
        sys.stderr.write(f"fake-mouser: {self.command} {urlparse(self.path).path}\n")

    def reply(self, body: dict, status: int = 200):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def body(self) -> dict:
        n = int(self.headers.get("Content-Length") or 0)
        return json.loads(self.rfile.read(n) or b"{}") if n else {}

    def route(self, method: str):
        u = urlparse(self.path)
        q = {k: v[0] for k, v in parse_qs(u.query).items()}
        if "apiKey" not in q:
            return self.reply({"Errors": [{"Code": "Invalid", "Message": "Invalid unique identifier."}]})
        p = u.path.removeprefix("/api").lower()
        b = self.body() if method == "POST" else {}
        if p == "/v1/search/partnumber":
            req = b.get("SearchByPartRequest", {})
            found = [m for n in req.get("mouserPartNumber", "").split("|") if (m := lookup(n))]
            return self.reply({"Errors": [], "SearchResults": {"NumberOfResult": len(found), "Parts": [part(m) for m in found]}})
        if p in ("/v1/search/keyword", "/v2/search/keyword"):
            req = b.get("SearchByKeywordRequest", {})
            in_stock = (req.get("searchOptions") or "").lower() in ("instock", "rohsandinstock")
            found = [
                m for m in PARTS
                if keyword_match(m, req.get("keyword") or "") and (not in_stock or int(PARTS[m][3]) > 0)
            ]
            shown = found[int(req.get("startingRecord") or 0):][: int(req.get("records") or 10) or 10]
            return self.reply({"Errors": [], "SearchResults": {"NumberOfResult": len(found), "Parts": [part(m) for m in shown]}})
        if p == "/v1/cart" and method == "GET":
            return self.reply(cart_reply(q.get("cartKey", "")))
        if p == "/v1/cart/item/remove":
            key, gone = q.get("cartKey", ""), lookup(q.get("mouserPartNumber", ""))
            CARTS[key] = [i for i in CARTS.get(key, []) if lookup(i["MouserPartNumber"]) != gone]
            return self.reply(cart_reply(key))
        if p == "/v1/cart/items/update":
            key = b.get("CartKey") or ""
            new = {lookup(i["MouserPartNumber"]): i for i in b.get("CartItems") or []}
            CARTS[key] = [new.get(lookup(i["MouserPartNumber"]), i) for i in CARTS.get(key, [])]
            return self.reply(cart_reply(key))
        if p in ("/v1/cart", "/v1/cart/items/insert"):
            key = b.get("CartKey") or str(uuid.uuid4())
            items = b.get("CartItems") or []
            CARTS[key] = items if p == "/v1/cart" else CARTS.get(key, []) + items
            return self.reply(cart_reply(key))
        if p == "/v1/order/options/query":
            return self.reply({"OrderInitialize": {
                "Errors": [], "CurrencyCode": "EUR", "BillingAddress": ADDRESS, "ShippingAddress": ADDRESS,
                "Shipping": {"Methods": [{"Code": c, "Method": m, "Rate": r} for c, m, r in SHIPPING]},
                "Payment": {"PaymentTypes": [{"Method": c, "Name": n} for c, n in PAYMENT]}}})
        if p == "/v1/order":
            o = b.get("Order") or b
            return self.reply(order(b, bool(o.get("SubmitOrder"))))
        return self.reply({"Errors": [{"Code": "NotFound", "Message": f"demo server: no {method} {p}"}]}, 404)

    def do_GET(self):
        self.route("GET")

    def do_POST(self):
        self.route("POST")


def main() -> int:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8787
    srv = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    sys.stderr.write(f"fake Mouser API on http://127.0.0.1:{port}/api (demo data, no real orders)\n")
    srv.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
