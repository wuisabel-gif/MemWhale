# Developer memory

Developer memory is evidence from software work that remains useful after the
session ends: the command that ran, its arguments and working directory, its
output and exit status, and a lesson about the fix that worked.

MemoryWhale focuses on development and debugging experience. It does not try to
store a person's entire life, replace source documentation, or act as an
autonomous agent. Its job is to make exact local evidence durable and
retrievable by both people and tools.

The product model has four parts:

1. [Capture](capture.md) records evidence.
2. [Memory](memory.md) preserves evidence and conclusions locally.
3. [Retrieval](retrieval.md) finds relevant failures and lessons.
4. [Interfaces](../architecture.md#4-interfaces) expose those capabilities to
   people and coding agents.
