#!/usr/bin/env python3
"""A stand-in for the Mouser API with fixed demo data, for recordings and trying the server.

    python3 demo/fake_mouser.py            # listens on 127.0.0.1:8787
    MOUSER_API_BASE=http://127.0.0.1:8787/api mouser-mcp

Nothing here talks to Mouser: searches answer from the parts below, carts live in memory,
and a submitted order gets a demo order number. Prices and stock are illustrative.
"""

import json
import sys
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

PARTS = {
    "ZX10R-2-183-S+": ("139-ZX10R-2-183-S+", "Mini-Circuits", "Signal Conditioning 2 Ways Resistive Power Splitter, DC - 18000 MHz, 50 Ohm", "2", "161,10 €", 161.10),
    "FMAT7467-6": ("67-FMAT7467-6", "Fairview Microwave", "Attenuators 6 dB Fixed Attenuator SMA Male to SMA Female DC to 12 GHz, 2 W", "6", "53,10 €", 53.10),
    "415-0033-M1.0": ("530-415-0033-M1.0", "Cinch Connectivity Solutions", "RF Cable Assemblies Straight SMA Plug to Straight SMA Plug, 1 m", "402", "17,44 €", 17.44),
    "415-0033-M2.0": ("530-415-0033-M2.0", "Cinch Connectivity Solutions", "RF Cable Assemblies Straight SMA Plug to Straight SMA Plug, 2 m", "372", "22,68 €", 22.68),
    "ST0645": ("67-ST0645", "Fairview Microwave", "RF Terminators SMA Male Termination 1 W to 6 GHz, VSWR 1.15", "104", "12,63 €", 12.63),
    "ZFRSC-42-S+": ("139-ZFRSC-42-S", "Mini-Circuits", "Signal Conditioning 2 Ways Resistive Power Splitter, DC - 4200 MHz, 50 Ohm", "0", "81,38 €", 81.38),
}
BY_MOUSER = {v[0].lower(): k for k, v in PARTS.items()}
SHIPPING = [(65, "UPS Worldwide Express Saver", 40.0), (21, "DHL International Express", 40.0)]
PAYMENT = [(1081, "PayPal")]
ADDRESS = {
    "AddressLocationTypeID": 1, "FirstName": "Ada", "LastName": "Lovelace", "Company": "Analytical Engines",
    "AddressOne": "Demo Str. 1", "City": "Berlin", "PostalCode": "10115", "CountryCode": "DE", "PhoneNumber": "0",
}
CARTS: dict[str, list[dict]] = {}


def part(mpn: str) -> dict:
    mouser, maker, desc, stock, price, _ = PARTS[mpn]
    return {
        "MouserPartNumber": mouser, "ManufacturerPartNumber": mpn, "Manufacturer": maker, "Description": desc,
        "AvailabilityInStock": stock, "Availability": f"{stock} In Stock", "AvailabilityOnOrder": [],
        "LeadTime": "119 Days" if stock == "0" else "0 Days", "LifecycleStatus": None, "Min": "1", "Mult": "1",
        "PriceBreaks": [{"Quantity": 1, "Price": price, "Currency": "EUR"}],
        "ProductDetailUrl": f"https://www.mouser.de/ProductDetail/{mouser}", "ROHSStatus": "RoHS Compliant",
    }


def lookup(number: str) -> str | None:
    n = number.strip()
    for mpn in PARTS:
        if mpn.lower() == n.lower():
            return mpn
    return BY_MOUSER.get(n.lower())


def line(item: dict) -> dict:
    mpn = lookup(item["MouserPartNumber"]) or item["MouserPartNumber"]
    mouser, maker, desc, stock, _, unit = PARTS.get(mpn, (item["MouserPartNumber"], "", "", "0", "", 0.0))
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
        "MerchandiseTotal": merch, "TaxAmount": 0.0, "OrderTotal": round(merch + rate, 2),
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
            kw = (b.get("SearchByKeywordRequest", {}).get("keyword") or "").lower()
            found = [m for m, v in PARTS.items() if any(w in v[2].lower() or w in m.lower() for w in kw.split())]
            return self.reply({"Errors": [], "SearchResults": {"NumberOfResult": len(found), "Parts": [part(m) for m in found]}})
        if p == "/v1/cart" and method == "GET":
            return self.reply(cart_reply(q.get("cartKey", "")))
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
