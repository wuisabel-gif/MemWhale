#!/usr/bin/env python3
"""Run unchanged Waku after serializing its optional MCP SDK's first import.

The pinned Waku bridge imports the SDK concurrently on two startup threads.
Preloading its public entry points avoids the observed cold-import race; this
is not a replacement bridge, model client, permission policy, or capture hook.
"""


def prepare_mcp():
    from mcp import ClientSession, StdioServerParameters  # noqa: F401


def main():
    prepare_mcp()
    from waku.__main__ import main as waku_main
    waku_main()


if __name__ == '__main__':
    main()
