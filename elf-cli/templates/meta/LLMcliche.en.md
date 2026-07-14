# LLMcliche: LLM Cliché Ban Reference

> **INFORMATIVE TRANSLATION — NOT OPERATIVE.**
> Authoritative source: `LLMcliche.md` (Korean). The AI agent operates from the Korean
> original, not this file; this English version is for human reading only.
> To customize project rules, edit `ProjectRule.md` (not this file). See
> `AI_PARA_Framework.md` §1.1.

> **Status**: this document is a **non-exhaustive, non-binding reference** that helps
> apply the *principle* of `EliRule.md §3 rule 12` (LLM Cliché Ban). **Not a fixed
> whitelist/blacklist** — an unlisted word is still a target if it reads as cliché, and a
> listed word is allowed when used with a precise technical meaning. The norm is EliRule's
> principle; this list is only examples. Models update often and their outputs overlap
> heavily, so do not use this as a forensic fingerprint.

Exclusion basis (summary): drop vague filler, overused register, and voiceless cliché
phrasing, and replace with **concrete, active, direct** statements. English output only
(Korean dry logs are unaffected; applies when English vocabulary/phrasing is mixed in).

---

## 1. Vocabulary clichés (by category, non-exhaustive)

| Category | Example vocabulary |
|---|---|
| Cliché verbs | represent · serve/stand/act as · delve · navigate · leverage · harness · showcase · underscore · foster · enhance · alleviate · unlock · elevate · streamline · spearhead · embark · garner · bolster · revolutionize · shed light on · (vague) yield · constitute (vague copula "X constitutes Y" — use is) · intensify (vague escalation — use increase; quantitative intensity: §4) · tie (A to B) (bonding cliché — use link) |
| Vague adjectives | crucial · vital · pivotal · essential · seamless · intricate · profound · paramount · nuanced · multifaceted · holistic · vibrant · innovative · cutting-edge · meticulous · (vague) robust · (vague) comprehensive · (vague) trivial |
| Cliché nouns | tapestry · landscape · realm · paradigm · testament · cornerstone · beacon · myriad · plethora · synergy · ecosystem · verdict (elevates an evaluation/check result into courtroom register — use result/pass-fail) · readout (vague nominalization of a measured result — use measurement/signal; hardware readout path: §4) · X family (non-academic metaphorical grouping of methods — use concrete terms such as methods; defined family terms: §4) · provenance (inflated nominalization of "source" — use source/origin; data-lineage standard use: §4) · dossier (non-academic elevation of a compilation — use record/summary) · caveat (cliché hedging noun for a limitation/note — use limitation/note) |
| Overused connectors | Moreover · Furthermore · Consequently · Additionally · sentence-initial runs of However · Notably · Importantly · Indeed · Nonetheless |
| Cliché phrases | "it is worth/important noting that" · "plays a crucial/pivotal role" · "pave the way" · "at the forefront of" · "in the realm of" · "a wide range/array of" · "rich tapestry" · "ever/rapidly evolving" · "revolutionize the way" · "in parallel (with)" (vague narrative linking) |
| Emphasis adverbs | uniquely · fundamentally · primarily · meticulously · crystallize |

- Metaphorical substitutions (lens · navigate · deep dive · journey · shed light on · crystallize) are governed by `EliRule §3 rule 3` (no metaphor) — this section centers on non-metaphor vocabulary/register.

## 2. Assistant register · conversational clichés (cross-model, non-exhaustive)

Register clichés common to chat/coding assistants (ChatGPT, Claude, Gemini, etc.).
Watch for them leaking into logs, documents, and commit messages.

| Type | Example |
|---|---|
| Opening greeting / over-agreement | "Certainly!" · "Absolutely!" · "Sure!" · "Great question" · "You're absolutely right" |
| Self-reference | "I'd be happy to help" · "Let me explain" · "Let me break this down" · "I hope this helps" · "feel free to" |
| Excess hedging/qualifiers | "It's worth noting that" · "It's important to note/remember" · "Generally speaking" · "In many cases" · "While this may vary" |
| Formulaic constructions | "not only … but also" · "It's not X, it's Y" (repeated within one piece) · closing "In conclusion / In summary" |
| Structural tic | compulsive rule-of-three lists · repeated "**Bold term**: explanation" one-line items · excessive bullets |
| Punctuation | em-dash (—) overused multiple times per paragraph (where a comma or parenthesis suffices) |

## 3. Model-leaning tendencies (non-deterministic, overlapping)

> Caution: outputs overlap heavily and change with each model update — these are
> *tendencies*, not an absolute classification.

| Model | Reported tendency | Evidence tier |
|---|---|---|
| GPT / ChatGPT | surging excess vocabulary (delve · underscore · meticulous · boast · intricate · tapestry); formal/clinical tone | academic (Kobak · FSU) |
| Claude / Claude Code | em-dash overuse; excess qualifiers/hedges; self-reference ("I'd be happy to help" · "Let me"); "**Bold**: explanation" lists; tangential ethics/generalization | community observation (non-academic) |
| Gemini | conversational/approachable plain style; excessive bullets; in long documents the requested voice drifts to an informational tone | reporting (Scientific American) |

## 4. Exceptions — precise technical use (allowed, non-exhaustive)

A word from the exclusion set is allowed when used with a **precise technical
denotation**. Examples:

| Word | Allowed context |
|---|---|
| robust / robustness | statistical/control robustness (robust estimator, etc.) |
| significance / significant | statistical significance (p-value) — vague emphasis like "significant improvement" is excluded |
| comprehensive | when it actually means exhaustive/full coverage |
| vital | standard terms such as vital sign (medicine) |
| essential | standard terms such as essential amino acid |
| yield | quantitative: yield strength · reaction yield · yield curve |
| trivial | definitional: trivial/non-trivial solution in math/CS |
| readout | hardware readout circuitry/signal path (detector readout, etc.) — for a general result/signal use measurement/signal |
| family | defined family terms such as a family of curves/distributions (math/statistics) or a protocol family (networking) — vague metaphorical groupings like "the X family of methods" are excluded |
| intensify | quantitative increase of a physical intensity — vague escalation/worsening is excluded |
| provenance | standard data-lineage use (data provenance, W3C PROV, reproducibility records) — for a general "source" use source/origin |

- **Domain homonyms** (e.g., energy/fitness `landscape`, experimental-psychology `paradigm`, `realm`) differ per project → **declare a carve-out in `ProjectRule.md`**. This document and EliRule keep only the general principle (no accumulation of per-project whitelists).

## 5. Before / After (pattern examples — domain-neutral)

| Before (LLM cliché) | After (concrete, active, direct) |
|---|---|
| X **represents** a major challenge | X **is** a major challenge |
| The method **provides crucial insights** | The method **measures** [concrete target/figure] |
| The signal is **uniquely governed** by geometry | The signal **is governed** by geometry |
| **By leveraging** the calibration | **Using** the calibration |
| The result is, **importantly**, **not trivial** | The effect **is 3× the baseline** (hedge removed, magnitude stated) |
| The checker returns its **verdict** | The checker **returns a result / passes or fails** |
| X **constitutes** a limitation that **intensifies** over time | X **is** a limitation that **increases** over time |
| **A caveat**: the dataset's **provenance** is unclear | **A limitation**: the dataset's **source** is unclear |

---

## References
- Norm (principle): `.elf/managed/EliRule.md` §3 rule 12 (LLM Cliché Ban) — this document is its applied examples.
- Metaphor exclusion: `EliRule.md` §3 rule 3.
- Per-project technical homonym exceptions: `ProjectRule.md`.
- Per-project vocabulary add/remove/override (overlay): `0_Meta/LLMcliche.project.md` — base ⊕ overlay; removals need a reason (EliRule §2.7).

## Sources
- Kobak et al., "Delving into LLM-assisted writing in biomedical publications through excess vocabulary," arXiv:2406.07016 (Science Advances, 2025) — 14M PubMed abstracts; excess words delve, underscore, meticulous, boast, etc.
- FSU, "Why Does ChatGPT 'Delve' So Much?" (Proc. COLING 2025) — causes of over-representation.
- Scientific American (2025), "ChatGPT and Gemini AI Have Uniquely Different Writing Styles."
- Caution: some per-model tells are community observations (non-academic tier), and this list is register-based and non-exhaustive. It shifts with new models and updates.
