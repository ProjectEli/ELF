# High-IF Journal Whitelist (External Source Retrieval)

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `highIFjournals.md` (Korean). The AI agent operates from the
> Korean original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See
> `AI_PARA_Framework.md` §1.1.

A domain-whitelist reference for literature search. Referenced from `EliRule.md §4`
(External Source Retrieval). Use: set the domains below as the web-search tool's
`allowed_domains` → surface high-tier journals first. Define the per-project priority
target subset (submission order, etc.) in each project's `ProjectRule.md`.
Per-project domain add/remove/override goes in the project overlay
`0_Meta/highIFjournals.project.md` (base ⊕ overlay; removals need a reason — EliRule §2.7).

## 1. Domain Whitelist (for `allowed_domains`)

| Domain | Journals included (representative) | Note |
|--------|------------------|------|
| `nature.com` | Nature, Nature Communications, Nature Materials/Photonics/Electronics/Methods/Medicine/Biotechnology/Nanotechnology, Nature Biomedical Engineering, Light: Science & Applications, Scientific Reports | full-text fetch often 403 → OA via PMC |
| `science.org` | Science, Science Advances, Science Robotics, Science Translational Medicine | |
| `cell.com` | Cell, Matter, Joule, Cell Reports, Med | |
| `pnas.org` | PNAS | |
| `onlinelibrary.wiley.com`, `advanced.onlinelibrary.wiley.com` | Advanced Materials, Advanced Science, Advanced Functional Materials, Advanced Energy Materials, Small, Laser & Photonics Reviews, InfoMat | |
| `pubs.acs.org` | ACS Nano, Nano Letters, JACS, Analytical Chemistry | |
| `opg.optica.org`, `optica.org` | Optica, Optics Express, Optics Letters, Advanced Photonics, Photonics Research, Biomedical Optics Express | optics-specialized |
| `ieeexplore.ieee.org` | IEEE TBME, JBHI, Sensors Journal, TMI, etc. | some abstract-only |
| `sciencedirect.com` | Biosensors and Bioelectronics, Biomaterials, etc. (Elsevier) | usually abstract-only |
| `pubs.rsc.org` | Lab on a Chip, Chemical Science, Energy & Environmental Science, Materials Horizons, Nanoscale, Chemical Society Reviews | RSC |
| `link.springer.com`, `*.springeropen.com` | eLight, PhotoniX, Nano-Micro Letters | Springer (non-nature.com) |
| `iopscience.iop.org` | Biofabrication, 2D Materials, Nanotechnology, Reports on Progress in Physics | IOP |
| `journals.aps.org` | Reviews of Modern Physics, Physical Review X, Physical Review Letters | APS (physics review/primary) |

## 2. OA Full-text Mirrors (403 bypass)

| Domain | Use |
|--------|------|
| `ncbi.nlm.nih.gov/pmc` | PMC — OA paper full text (mirrors NatComm, SciAdv, AdvSci, LSA, etc.) |
| `europepmc.org` | Europe PMC — OA full text + all PMC abstracts |
| `arxiv.org`, `biorxiv.org`, `medrxiv.org` | preprint full text |

## 3. Bibliometric APIs (IF-proxy sorting/validation)

| API | endpoint | Provides |
|-----|----------|------|
| OpenAlex | `api.openalex.org` | cited_by_count, source venue (no key) |
| Semantic Scholar | `api.semanticscholar.org/graph/v1` | citations, influential citations |
| Crossref | `api.crossref.org` | DOI metadata, citation count |

## 4. Usage Notes

- "Search exposure ≠ high quality" — before citing, cross-check venue + citation count via the §3 APIs.
- When you must forcibly exclude low-IF OA (MDPI, Frontiers, Hindawi, etc.), use the search tool's `blocked_domains`. Beware of excluding legitimate venues alongside — do not exclude unconditionally.
- This list is a general reference (multidisciplinary, regardless of field, including optics/device/bio high-IF). Define the per-project target subset and submission order in `ProjectRule.md`.
- When updating the list, edit only this file — rule documents reference this file, so no extra synchronization is needed.
