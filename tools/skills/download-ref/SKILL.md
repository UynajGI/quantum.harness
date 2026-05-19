---
name: download-ref
description: Use when adding arXiv IDs, DOIs, or bibliography stubs to this quantum many-body harness. Stores rendered markdown under knowledge-base/literature/<method>/ and keeps raw PDFs, metadata, and extracted figures gitignored inside each method folder.
---

# download-ref

Add methodology references to this repo's knowledge base.

This is the repo-local adaptation of the `zlp-harness` download workflow. The
upstream workflow assumes a `.knowledge/` root; this harness uses
`knowledge-base/` and organizes methodology references by method.

## Layout

For method slug `<method>`:

```text
knowledge-base/literature/<method>/
  INDEX.md
  <rendered-reference>.md
  .raw/      # metadata + PDFs, gitignored
  .figures/  # extracted PDF images, gitignored
```

Use method slugs that match the local method cards when possible:

```text
dmrg
tebd
ed
vmc-nqs
anderson-impurity
spectral
finite-t
dmft
qmc
```

## Scripts

The helper scripts are bundled with this skill:

```sh
SCRIPTS="$(pwd)/tools/skills/download-ref/scripts"
```

They are idempotent. Re-running a manifest skips existing metadata/PDFs and
overwrites rendered markdown from raw sources.

## Workflow

### 1. Prepare a method folder

```sh
METHOD=dmrg
KB="$(pwd)/knowledge-base/literature/$METHOD"
mkdir -p "$KB"
```

### 2. Build a manifest

Use arXiv IDs without `arXiv:` and without version suffixes. Old-style IDs such
as `cond-mat/0701105` are supported. DOIs may be given verbatim. Stubs are for
books or closed references that should be indexed but cannot be downloaded.

```json
{
  "arxiv": ["1008.3477"],
  "doi": ["10.1007/BFb0106062"],
  "stub": [
    {
      "slug": "hewson-1993-kondo-problem",
      "title": "The Kondo Problem to Heavy Fermions",
      "authors": "A. C. Hewson",
      "year": "1993",
      "note": "Canonical book reference; publisher access required."
    }
  ]
}
```

### 3. Fetch metadata and PDFs

```sh
python3 "$SCRIPTS/fetch_metadata.py" \
  --kb "$KB" \
  --manifest "$MANIFEST" \
  --download-arxiv-pdfs
```

The helper writes to `$KB/.raw/`. For DOI references, it tries Semantic Scholar
metadata and uses an arXiv preprint when one is available.

### 4. Render markdown

```sh
python3 "$SCRIPTS/render.py" \
  --kb "$KB" \
  --manifest "$MANIFEST"
```

Rendering uses `pymupdf4llm` when installed, then falls back to `markitdown` or
`pdftotext`. If `pymupdf4llm` is missing, full text can still render but figures
may be absent.

For a single PDF that is already in hand, render it with the same extraction
stack:

```sh
python3 "$SCRIPTS/render.py" \
  --pdf sources/paper.pdf \
  --out sources/paper.md
```

For long lecture notes or books where image/OCR extraction is too slow, prefer
searchable text-only rendering:

```sh
python3 "$SCRIPTS/render.py" \
  --kb "$KB" \
  --manifest "$MANIFEST" \
  --text-only
```

### 5. Regenerate the method index

```sh
python3 "$SCRIPTS/index.py" \
  --kb "$KB" \
  --title "$METHOD methodology references" \
  --source-note "Methodology references for the quantum many-body physics harness. Raw PDFs and extracted figures are local-only and gitignored."
```

Keep the title/source-note stable for repeat runs in the same method folder.

### 6. Verify

```sh
git check-ignore "$KB/.raw/" "$KB/.figures/" || true
test -f "$KB/INDEX.md"
find "$KB" -maxdepth 1 -name '*.md' -print
```

Report the rendered files, which entries have `full_text: yes`, and which are
metadata-only or stubs.

## Notes

- Do not commit `.raw/` or `.figures/`.
- Do not put all methods in one folder; use one method folder per methodology.
- For paywalled books, create a stub entry rather than fabricating a PDF.
