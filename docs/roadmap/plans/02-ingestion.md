# Plan 02 — Ingestion ⭐ (the lean digestion path)

**Crate:** `cite-lint-ingest` (offline) · **Milestone:** M2 (matures ongoing) ·
**Depends on:** 00, 01 · **Size:** L

> This is the layer the brief calls out: *ingest unstructured, rule-heavy documents like
> AGLC and convert them into highly performant, parseable, lintable rule sets — even if
> it's LLM-assisted.* The design principle that makes that safe:
> **LLMs do the toil; deterministic gates own the truth.** No LLM output reaches the
> committed rule set without passing a verification oracle.

## Goal & Definition of Done

Turn the AGLC4 PDF (336 pages, bookmarked, AES-encrypted/empty-password, copy-restricted)
into the committed `data/editions/aglc4/{tables,rules}` of plan 01 — *quickly, auditably,
and reproducibly* — and make the next edition a **diff + review**, not a re-read.

**DoD**
- [ ] One command runs the whole pipeline: `acquire → decrypt → segment → extract →
      verify → review → embed`.
- [ ] LLM-assisted extraction of vocab tables **and** draft Rule IR, every item
      provenance-tagged to a PDF page.
- [ ] A **deterministic verification oracle** that must pass before anything is written to
      `data/` — including the guide's own worked examples linting clean.
- [ ] Re-running ingestion reproduces the hand-authored case tables (plan 01 T5) within
      *reviewed* differences (no silent change).
- [ ] `diff <old> <new>` emits an edition changelog for human/LLM review.
- [ ] The PDF, page images, and embeddings are **never committed** (CI-guarded); only
      facts + provenance + changelogs are.
- [ ] CI tests run on a committed **fixture excerpt**, never the full PDF.

## Design context

[`architecture.md`](../architecture.md) §5 (Rule IR pipeline) and design doc §7. Enforcement
is deterministic off committed tables/IR (P3); the vector store and any LLM are
**authoring-only** and never on the lint hot path. Writes land in plan 01's schema.

## Research spikes (the heart of "make it lean")

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-ING-1](../research/README.md#r-ing-1) | Extraction modality: text-layer vs page-image **vision** vs hybrid — which extracts the appendix tables most faithfully? Bake-off on one reporters page. | 2 days | T5 |
| [R-ING-2](../research/README.md#r-ing-2) | Verification strategy: do the worked-example oracle + dual-model agreement give enough confidence to auto-accept high-confidence rows and human-review only the rest? Set thresholds. | 2 days | T7, T8 |
| [R-ING-3](../research/README.md#r-ing-3) | Lean, memory-safe decrypt+extract toolchain (qpdf/pikepdf decrypt; pdfium/pdftotext/render). | 1 day | T2 |
| [R-ING-4](../research/README.md#r-ing-4) | Local vector store (sqlite-vec vs Lance) + local embedder — offline, small. | 1 day | T9 |
| [R-ING-5](../research/README.md#r-ing-5) | Edition-diff matching: embedding-NN + structural bookmark alignment. | 1 day | T10 |
| [R-ING-6](../research/README.md#r-ing-6) | LLM Rule-IR drafting: schema-constrained output, hallucination guards, auto-draftable vs human-only split. | 2 days | T6 |

## Pipeline (stages)

```
acquire → decrypt+extract → segment → classify → ┬─ LLM table extract ─┐
                                                  └─ LLM rule draft ────┴─▶ VERIFY (oracle) ─▶ review ─▶ data/
                                                                              │
                                              embed (local vector store, git-ignored) ─▶ edition-diff
```

## Task ladder

- **T1 — Acquire.** Wrap the existing [`scripts/fetch-source.sh`](../../../scripts/fetch-source.sh)
  as `cite-lint-ingest acquire [edition]`: fetch to git-ignored `.cache/`, verify the pinned
  `meta.toml` SHA256, TOFU-pin on first fetch from a permitted network. *Check:* exits ok on
  a matching cache; errors on SHA mismatch. *(Design risk #1 already mitigated by the script.)*
- **T2 — Decrypt + extract.** Decrypt the empty-password AES PDF (toolchain from R-ING-3),
  then extract per-page **text** and render per-page **images**, retaining page/coords for
  provenance. Resource-limit the parse (size/time caps) and use memory-safe libs (no shelling
  to untrusted converters). *Check:* page count == `meta.toml` `pages` (336); text+image for a
  known page.
- **T3 — Segment by bookmarks.** Parse the PDF outline; split into addressable sections keyed
  by rule number / appendix + page span. *Check:* section count stable; a known rule (e.g. the
  case-citation rule) maps to its expected page span.
- **T4 — Classify sections.** Label each section `table | prose-rule | example | other` (the
  appendix tables vs rule prose vs worked examples). *Check:* the reporters appendix classifies
  as `table`; the worked-examples section as `example` (feeds the oracle, T7).
- **T5 — LLM table extraction.** For each `table` section, prompt an LLM with the section
  **text + page image** and a **strict JSON schema**; get rows → candidate `tables/*.toml`
  with `provenance{page,...}`. Run **two independent passes** (different prompt/model);
  agreement → high-confidence, disagreement → flagged. *Check:* extracted reporters include
  `CLR` with round-year + High Court; disagreements surface in the report.
- **T6 — LLM rule drafting.** For each `prose-rule` section, prompt an LLM to draft a Rule IR
  entry (trigger/predicate/message/`aglc_ref`/fix) **or** mark it `algorithmic`/`low-confidence`
  for a human. Schema-constrained output (must validate against plan 01 T2). *Check:* a drafted
  rule validates and carries an `aglc_ref` to the correct page.
- **T7 — Verification oracle (the trust gate).** Nothing reaches `data/` until **all** pass:
  1. **Schema + validation** via plan 01's loader (dangling vocab refs, missing provenance,
     duplicate codes → reject).
  2. **Worked-example oracle:** load the candidate tables/IR into `core` and lint the guide's
     own worked examples (from T4 `example` sections) — compliant examples must pass; machine-
     mutated variants must fail. *This is the strongest signal: the guide validates its own
     extraction.*
  3. **Round-trip diff:** re-render each extracted row and diff against the source text region;
     numeric/string fidelity check.
  4. **Dual-model agreement** (from T5/T6): below the R-ING-2 threshold → route to review.
  5. **Provenance present** on every row/rule.
  Emit a `verification-report.md` (pass/fail per check, with page refs). *Check:* a deliberately
  corrupted row fails check 2 or 3 and is blocked.
- **T8 — Human review gate.** `cite-lint-ingest review` shows **only** low-confidence/changed
  items with their provenance + source snippet for accept/edit/reject. High-confidence,
  oracle-passing items can auto-stage (threshold from R-ING-2). *Check:* a flagged disagreement
  cannot be committed without an explicit accept.
- **T9 — Embed (authoring-only).** Embed each section into a **local** vector store
  (R-ING-4), git-ignored build artifact. Used for edition-diff (T10), authoring retrieval, and
  dev-loop memory (plan 10). **Never read at lint time** — guarded by the dep-graph test (P3).
  *Check:* store builds; a dep test proves `core` cannot reach it.
- **T10 — Edition-diff.** `cite-lint-ingest diff <old> <new>`: embed the new edition, match
  prior↔new sections (embedding-NN + bookmark alignment, R-ING-5), emit a changelog of
  added/removed/changed sections. *Check:* on synthetic before/after sections, a changed rule
  shows up as `changed` with both page refs.
- **T11 — Fixture-excerpt CI tests.** Commit a **small, non-infringing fixture** (synthetic
  AGLC-shaped pages, *not* the real PDF) exercising segment → extract → verify → diff
  deterministically. *Check:* CI runs the pipeline on the fixture without network or the real PDF.
- **T12 — Audit guards.** Assert in CI that `*.pdf`, page images, and the vector store are
  git-ignored and absent from the tree; every emitted row/rule has provenance. *Check:* adding a
  `.pdf` under `data/` or `.cache/` to the index fails CI.

## Acceptance gate

`cite-lint-ingest run aglc4` over the real (cached) PDF reproduces plan 01's case tables within
reviewed differences; the verification oracle (esp. the worked-example check) passes; the
fixture-excerpt pipeline is green in CI with no network and no real PDF; `diff` produces a
changelog; PDF/images/embeddings are provably never committed.

## Lean notes (this is where leanness is won)

- **No bespoke geometric table parsers.** LLM extraction + the deterministic oracle is far
  less code and more robust to AGLC's varied table layouts than hand-rolled column detection.
- **The guide verifies itself.** Worked examples are a free, high-signal oracle — we don't
  hand-write a giant expected-tables golden; we assert known anchors (plan 01) + let the oracle
  cover breadth.
- **One vector store, two uses** — edition-diff *and* dev-loop memory (plan 10) share it.
- **Offline/local-first.** Prefer a local embedder and (where feasible) a local LLM; if a
  hosted LLM is used, send only factual table/rule **regions**, never commit prose/embeddings,
  and document the data-handling (P7, plan 08). Keeps copyright + confidentiality clean.
- **Commit facts, not the work product:** tables + Rule IR + provenance + changelogs only.

## Risks & mitigations

- *Extraction fidelity* (design risk #2) → worked-example oracle + dual-model agreement +
  round-trip diff + human gate + plan 01 table-tests. Multiple independent checks, not trust.
- *LLM hallucinating a rule* → schema-constrained drafting + `verify-aglc-ref` cross-check
  (the cited page must actually contain the rule) + oracle + human gate. *Never* a confident
  wrong rule (correctness guardrail).
- *Copyright / confidentiality* (design risk #5) → PDF, images, embeddings never committed;
  only factual tables + changelogs; documented data-handling for any hosted model.
- *WAF-blocked fetch* (design risk #1) → existing curl-with-headers + Playwright fallback +
  TOFU pin.
- *Pipeline drift over editions* → the diff + review gate + table-tests make every edition
  change explicit and reviewed.
