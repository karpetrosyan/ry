<div align="center">
  <img src=".github/logo.png" alt="ry logo" width="600"/>
</div>

**ry** stands for "repeat yourself" - the opposite of DRY.

Inspired by [unasync](https://github.com/python-trio/unasync), but built from the ground up for inline transformations and more flexible configurations.

Ry transforms the Python code using regex rules. Originally built to simplify maintaining dual async/sync codebases, but works for any pattern-based transformations.

Comes with a package system where you can define reusable sets of rules. Have a built-in `std` package with all standard rules (like `async def` to `def`, etc), and you can create your own packages for your specific transformations.

## Inline Transformations

Ry can transform not only whole files, but also specific lines or blocks of code based on inline comments.

```diff
class Response:
  
    async def aclose(self) -> None:  # unasync: generate
        if self._stream_to_close is None:
            return

        if not isinstance(self._stream_to_close, AsyncClosableStream):
            raise AsyncSyncMismatchError("Can't call `aclose` in this context")

        await self._stream_to_close.aclose()
        self._stream_to_close = None

+    def close(self) -> None:  # unasync: generated
+        if self._stream_to_close is None:
+            return
+
+        if not isinstance(self._stream_to_close, AsyncClosableStream):
+            raise AsyncSyncMismatchError("Can't call `close` in this context")
+
+        self._stream_to_close.close()
+        self._stream_to_close = None

```


## Usage

Validate without modifying files:

```bash
ry src/
```

Fixes all the fixable issues:

```bash
ry --fix src/
```
