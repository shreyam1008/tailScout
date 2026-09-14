"""Verify the published site identity and its crawlable raster favicon."""
import json
import struct
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse

class Head(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links, self.meta, self.schemas = [], {}, []
        self.ld = None
    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "link": self.links.append(a)
        if tag == "meta": self.meta[a.get("property", a.get("name"))] = a.get("content")
        if tag == "script" and a.get("type") == "application/ld+json": self.ld = ""
    def handle_data(self, data):
        if self.ld is not None: self.ld += data
    def handle_endtag(self, tag):
        if tag == "script" and self.ld is not None:
            self.schemas.append(json.loads(self.ld))
            self.ld = None

def nodes(value):
    if isinstance(value, dict):
        if value.get("@type") == "WebSite": yield value
        for child in value.values(): yield from nodes(child)
    elif isinstance(value, list):
        for child in value: yield from nodes(child)

root, name, url = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
p = Head()
p.feed((root / "index.html").read_text(encoding="utf-8"))
sites = list(nodes(p.schemas))
assert len(sites) == 1 and sites[0].get("name") == name, sites
assert sites[0].get("url", "").rstrip("/") == url.rstrip("/"), sites
assert p.meta.get("og:site_name") == name, p.meta
assert any(x.get("rel") == "canonical" and x.get("href", "").rstrip("/") == url.rstrip("/") for x in p.links)
icons = [x for x in p.links if "icon" in x.get("rel", "").split() and urlparse(x.get("href", "")).path.endswith(".png")]
assert icons, "Missing PNG favicon link"
for icon in icons:
    path = root / urlparse(icon["href"]).path.lstrip("/")
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", path
    width, height = struct.unpack(">II", data[16:24])
    assert width == height and width > 48, (path, width, height)
    if icon.get("sizes"): assert icon["sizes"] == f"{width}x{height}"
print(f"Verified {name}: canonical name and square PNG favicon")
