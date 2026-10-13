#!/usr/bin/env python3
"""Fails when a relative Markdown link in the repository points at a missing file."""

import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LINK = re.compile(r"\]\(([^)\s]+)\)")


def markdown_files():
    listed = subprocess.run(
        ["git", "ls-files", "*.md"], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return [os.path.join(ROOT, name) for name in listed.stdout.split() if name]


def broken_links():
    bad = []
    for path in markdown_files():
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        for match in LINK.finditer(text):
            target = match.group(1)
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            target = target.split("#", 1)[0]
            if not target:
                continue
            resolved = os.path.normpath(os.path.join(os.path.dirname(path), target))
            if not os.path.exists(resolved):
                bad.append((os.path.relpath(path, ROOT), target))
    return bad


def main():
    bad = broken_links()
    for source, target in bad:
        print(f"{source}: broken link to {target}")
    if bad:
        return 1
    print("all relative links resolve")
    return 0


if __name__ == "__main__":
    sys.exit(main())
