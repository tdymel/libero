# Vendored axe-core

`axe.zip` holds **axe-core 4.10.2** (`axe.min.js`) and its licence
(`LICENSE-MPL-2.0.txt`), which is **MPL-2.0**. The licence travels inside the
archive because MPL-2.0 requires the notice to accompany the code.

The runner extracts both files into this directory on first use. They are
gitignored, so only the archive is committed.

**After extraction the licence is at `e2e/vendor/LICENSE-MPL-2.0.txt`.** Anyone
auditing the repo's licences without running the suite can read it out of the
archive directly:

```
unzip -p e2e/vendor/axe.zip LICENSE-MPL-2.0.txt
```

axe is test-only. Nothing in `libero` or `docs` references it and it is not
shipped.
