#!/usr/bin/env python3
"""Check and update localized README files from README.md.

The script deliberately uses only the Python standard library so it can run in
GitHub Actions without installing a translation SDK. Translation uses an
OpenAI-compatible chat-completions endpoint and returns a complete Markdown
file. Protected Markdown, HTML, and code tokens are validated before a
translated file is written.
"""
from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "README.md"
LOCALES = {
    "zh-CN": ROOT / "README.zh-CN.md",
    "zh-TW": ROOT / "README.zh-TW.md",
    "ko": ROOT / "README.ko.md",
    "ja": ROOT / "README.ja.md",
}
MARKER_RE = re.compile(r"^<!-- memorywhale-i18n-source-sha: ([0-9a-f]{64}) -->\s*$", re.MULTILINE)
FENCE_RE = re.compile(r"```[^\n]*\n.*?```", re.DOTALL)
URL_RE = re.compile(r"https?://[^\s)>'\"]+")
LINK_TARGET_RE = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
INLINE_CODE_RE = re.compile(r"(?<!`)`([^`\n]+)`(?!`)")
HTML_TAG_RE = re.compile(r"</?([A-Za-z][A-Za-z0-9-]*)\b[^>]*>")
HTML_ATTR_RE = re.compile(r"\b(?:href|src)\s*=\s*[\"']([^\"']+)[\"']")
HEADING_RE = re.compile(r"^(#{1,6})\s+", re.MULTILINE)


def source_hash() -> str:
    return hashlib.sha256(SOURCE.read_bytes()).hexdigest()


def read_locale(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def locale_hash(text: str) -> str | None:
    match = MARKER_RE.search(text)
    return match.group(1) if match else None


def stamp(text: str, digest: str) -> str:
    text = MARKER_RE.sub("", text, count=1).lstrip("\n")
    return f"<!-- memorywhale-i18n-source-sha: {digest} -->\n\n{text}"


def protected_tokens(text: str) -> tuple[list[str], ...]:
    return (
        FENCE_RE.findall(text),
        URL_RE.findall(text),
        LINK_TARGET_RE.findall(text),
        INLINE_CODE_RE.findall(text),
        HTML_TAG_RE.findall(text),
        HTML_ATTR_RE.findall(text),
    )


def validate_translation(source: str, translated: str, locale: str) -> None:
    source_tokens = protected_tokens(source)
    translated_tokens = protected_tokens(translated)
    token_names = (
        "code block",
        "URL",
        "link target",
        "inline code",
        "HTML tag",
        "HTML link target",
    )
    changed = [
        name
        for name, expected, actual in zip(token_names, source_tokens, translated_tokens)
        if Counter(expected) != Counter(actual)
    ]
    if changed:
        raise ValueError(
            f"translation for {locale} changed protected content: {', '.join(changed)}"
        )
    if HEADING_RE.findall(source) != HEADING_RE.findall(translated):
        raise ValueError(f"translation for {locale} changed the README heading structure")
    if not HEADING_RE.search(translated):
        raise ValueError(f"translation for {locale} is not a Markdown document")


def translate(source: str, existing: str, locale: str) -> str:
    api_key = os.environ.get("OPENAI_API_KEY", "").strip()
    if not api_key:
        raise RuntimeError("OPENAI_API_KEY is not configured")
    endpoint = (os.environ.get("OPENAI_BASE_URL") or "https://api.openai.com/v1").rstrip("/")
    model = os.environ.get("OPENAI_MODEL") or "gpt-4o-mini"
    prompt = f"""You maintain the {locale} translation of a public technical README.
Update the existing translation using the English source below. Return only the
complete translated Markdown document, without an outer code fence or commentary.

Rules:
- preserve the exact heading structure and order;
- preserve every fenced code block byte-for-byte, including commands;
- preserve every URL, image path, badge, HTML tag, CLI flag, file path, and product name;
- translate prose naturally for {locale};
- do not translate code, shell commands, URLs, or identifiers;
- keep the document useful to a technical reader.

ENGLISH SOURCE:
{source}

EXISTING {locale} TRANSLATION:
{existing}
"""
    payload = json.dumps({
        "model": model,
        "temperature": 0.1,
        "messages": [
            {"role": "system", "content": "You are a precise technical documentation translator."},
            {"role": "user", "content": prompt},
        ],
    }).encode("utf-8")
    request = urllib.request.Request(
        f"{endpoint}/chat/completions",
        data=payload,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            result = json.load(response)
    except urllib.error.HTTPError as error:
        detail = error.read().decode("utf-8", errors="replace")[:500]
        raise RuntimeError(f"translation provider returned HTTP {error.code}: {detail}") from error
    except urllib.error.URLError as error:
        raise RuntimeError(f"translation provider request failed: {error.reason}") from error
    try:
        content = result["choices"][0]["message"]["content"]
    except (KeyError, IndexError, TypeError) as error:
        raise RuntimeError("translation provider returned no chat completion content") from error
    if not isinstance(content, str) or not content.strip():
        raise RuntimeError("translation provider returned an empty translation")
    content = content.strip()
    if content.startswith("```markdown\n") and content.endswith("\n```"):
        content = content[len("```markdown\n") : -len("\n```")]
    return content


def check(digest: str) -> int:
    stale = False
    for locale, path in LOCALES.items():
        if not path.exists():
            print(f"{locale}: missing {path}")
            stale = True
            continue
        found = locale_hash(read_locale(path))
        if found == digest:
            print(f"{locale}: up to date")
        else:
            print(f"{locale}: stale (source {digest[:12]}, locale {(found or 'missing')[:12]})")
            stale = True
    return 1 if stale else 0


def stamp_existing(digest: str) -> None:
    source = SOURCE.read_text(encoding="utf-8")
    for locale, path in LOCALES.items():
        if not path.exists():
            raise RuntimeError(f"{locale}: missing {path}")
        existing = read_locale(path)
        validate_translation(source, existing, locale)
        path.write_text(stamp(existing, digest), encoding="utf-8")
        print(f"{locale}: stamped {digest}")


def translate_stale(digest: str) -> int:
    source = SOURCE.read_text(encoding="utf-8")
    changed = 0
    for locale, path in LOCALES.items():
        existing = read_locale(path) if path.exists() else ""
        if locale_hash(existing) == digest:
            print(f"{locale}: up to date")
            continue
        translated = translate(source, existing, locale)
        validate_translation(source, translated, locale)
        path.write_text(stamp(translated, digest), encoding="utf-8")
        print(f"{locale}: translated and stamped {digest}")
        changed += 1
    return changed


def main() -> int:
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true")
    group.add_argument("--stamp-existing", action="store_true")
    group.add_argument("--translate-stale", action="store_true")
    args = parser.parse_args()
    digest = source_hash()
    try:
        if args.check:
            return check(digest)
        if args.stamp_existing:
            stamp_existing(digest)
            return 0
        changed = translate_stale(digest)
        print(f"changed locales: {changed}")
        return 0
    except (OSError, RuntimeError, ValueError) as error:
        print(f"readme locale sync failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
