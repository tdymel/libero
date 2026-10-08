#!/usr/bin/env python3
"""Writes `<route>.html` next to a built `index.html` for every docs route.

Pages has no SPA rewrite, so a deep link used to answer 404 from `404.html`.
Pages serves `/buttons/button` from `buttons/button.html` with 200, and each
copy carries its page's title, description, canonical, Open Graph and Twitter
tags, a link to the markdown mirror, a `BreadcrumbList` and a `<noscript>`
summary. The rows come from `route_pages.tsv`, which `cargo test -p docs
route_pages` keeps current.

Run it after `index.html` is copied to `404.html`: the `/` row rewrites
`index.html` itself, and the 404 page must keep the site-wide head.
"""

import html
import json
import os
import re
import sys

SITE = "https://libero-ui.dev"
MANIFEST = os.path.join(os.path.dirname(os.path.abspath(__file__)), "route_pages.tsv")


def set_meta(page, key, value):
    pattern = re.compile(r'(<meta (?:name|property)="%s" content=")[^"]*(")' % re.escape(key))
    page, count = pattern.subn(lambda m: m.group(1) + html.escape(value) + m.group(2), page)
    if count != 1:
        sys.exit(f"route_pages.py: index.html has {count} <meta> for {key}")
    return page


def breadcrumbs(path, title, group, first):
    crumbs = [("Libero", SITE + "/")]
    # The section has no page of its own: its first page stands in, unless that is this one.
    if group and first != path:
        crumbs.append((group, SITE + first))
    crumbs.append((title, SITE + path))
    items = [
        {"@type": "ListItem", "position": i + 1, "name": name, "item": url}
        for i, (name, url) in enumerate(crumbs)
    ]
    data = {"@context": "https://schema.org", "@type": "BreadcrumbList", "itemListElement": items}
    # `</` would end the script element early.
    return json.dumps(data, ensure_ascii=False).replace("</", "<\\/")


def render(shell, base, row):
    path, title, description, markdown, group, first = row
    page_title = title.removesuffix(" - Libero")
    url = SITE + path
    page = re.sub(r"<title>[^<]*</title>", lambda _: f"<title>{html.escape(title)}</title>", shell, count=1)
    for key, value in [
        ("description", description),
        ("og:title", title),
        ("og:description", description),
        ("twitter:title", title),
        ("twitter:description", description),
    ]:
        page = set_meta(page, key, value)
    head = [
        f'<link rel="canonical" href="{html.escape(url)}">',
        f'<meta property="og:url" content="{html.escape(url)}">',
        f'<link rel="alternate" type="text/markdown" href="{html.escape(SITE + markdown)}">',
    ]
    if path != "/":
        head.append(
            '<script type="application/ld+json">'
            + breadcrumbs(path, page_title, group, first)
            + "</script>"
        )
    page = page.replace("</title>", "</title>\n        " + "\n        ".join(head), 1)
    noscript = (
        "<noscript>\n"
        f"            <h1>{html.escape(page_title)}</h1>\n"
        f"            <p>{html.escape(description)}</p>\n"
        "            <p>These docs need JavaScript and WebAssembly. This page is also plain\n"
        f'               markdown: <a href="{html.escape(base + markdown)}">{html.escape(page_title)} as markdown</a>.</p>\n'
        "        </noscript>"
    )
    page, count = re.subn(r"<noscript>.*?</noscript>", lambda _: noscript, page, count=1, flags=re.S)
    if count != 1:
        sys.exit("route_pages.py: index.html has no <noscript>")
    return page


def main(public):
    index = os.path.join(public, "index.html")
    with open(index, encoding="utf-8") as f:
        shell = f.read()
    if not os.path.exists(os.path.join(public, "404.html")):
        sys.exit("route_pages.py: copy index.html to 404.html first")
    # The shell's markdown link already carries the build's base path.
    found = re.search(r'href="([^"]*)/md/index\.md"', shell)
    if not found:
        sys.exit("route_pages.py: index.html has no link to /md/index.md")
    base = found.group(1)
    site_description = re.search(r'<meta name="description" content="([^"]*)"', shell).group(1)

    with open(MANIFEST, encoding="utf-8") as f:
        rows = [line.rstrip("\n").split("\t") for line in f if line.strip()]
    paths = {row[0] for row in rows}
    for row in rows:
        row[2] = row[2] or html.unescape(site_description)
        page = render(shell, base, row)
        path = row[0]
        targets = [index] if path == "/" else [os.path.join(public, path.lstrip("/") + ".html")]
        # A page that is also a directory (`/hooks`, `/hooks/use-drag`): should Pages
        # redirect to `/hooks/`, `hooks/index.html` answers it.
        if path != "/" and any(p.startswith(path + "/") for p in paths):
            targets.append(os.path.join(public, path.lstrip("/"), "index.html"))
        for target in targets:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, "w", encoding="utf-8") as f:
                f.write(page)
    print(f"route_pages.py: wrote {len(rows)} routes into {public}")


if __name__ == "__main__":
    main(sys.argv[1])
