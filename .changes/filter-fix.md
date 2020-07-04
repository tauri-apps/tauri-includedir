---
"codegen": patch
---

Fixes the `filter` param usage on the `build` function. Previously, when the `filter` was empty, all file entries would be ignored.
