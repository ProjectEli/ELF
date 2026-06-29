# High-IF Journal Whitelist (External Source Retrieval)

문헌 검색 시 도메인 화이트리스트 reference. `EliRule.md §4` (External Source Retrieval) 에서 참조.
용도: 웹 검색 도구의 `allowed_domains` 에 아래 도메인 지정 → high-tier 저널 우선 surfacing.
프로젝트별 우선 target subset(제출 순서 등)은 각 프로젝트 `ProjectRule.md` 정의.

## 1. 도메인 화이트리스트 (`allowed_domains` 용)

| 도메인 | 포함 저널 (대표) | 비고 |
|--------|------------------|------|
| `nature.com` | Nature, Nature Communications, Nature Materials/Photonics/Electronics/Methods/Medicine/Biotechnology/Nanotechnology, Nature Biomedical Engineering, Light: Science & Applications, Scientific Reports | 본문 fetch 403 빈번 → OA는 PMC 경유 |
| `science.org` | Science, Science Advances, Science Robotics, Science Translational Medicine | |
| `cell.com` | Cell, Matter, Joule, Cell Reports, Med | |
| `pnas.org` | PNAS | |
| `onlinelibrary.wiley.com`, `advanced.onlinelibrary.wiley.com` | Advanced Materials, Advanced Science, Advanced Functional Materials, Advanced Energy Materials, Small, Laser & Photonics Reviews, InfoMat | |
| `pubs.acs.org` | ACS Nano, Nano Letters, JACS, Analytical Chemistry | |
| `opg.optica.org`, `optica.org` | Optica, Optics Express, Optics Letters, Advanced Photonics, Photonics Research, Biomedical Optics Express | 광학 전문 |
| `ieeexplore.ieee.org` | IEEE TBME, JBHI, Sensors Journal, TMI 등 | 일부 abstract-only |
| `sciencedirect.com` | Biosensors and Bioelectronics, Biomaterials 등 (Elsevier) | 대개 abstract-only |
| `pubs.rsc.org` | Lab on a Chip, Chemical Science, Energy & Environmental Science, Materials Horizons, Nanoscale, Chemical Society Reviews | RSC |
| `link.springer.com`, `*.springeropen.com` | eLight, PhotoniX, Nano-Micro Letters | Springer (nature.com 외) |
| `iopscience.iop.org` | Biofabrication, 2D Materials, Nanotechnology, Reports on Progress in Physics | IOP |
| `journals.aps.org` | Reviews of Modern Physics, Physical Review X, Physical Review Letters | APS (물리 종설·원저) |

## 2. OA full-text 미러 (403 우회)

| 도메인 | 용도 |
|--------|------|
| `ncbi.nlm.nih.gov/pmc` | PMC — OA 논문 full text (NatComm·SciAdv·AdvSci·LSA 등 미러) |
| `europepmc.org` | Europe PMC — OA full text + 전 PMC abstract |
| `arxiv.org`, `biorxiv.org`, `medrxiv.org` | preprint full text |

## 3. bibliometric API (IF proxy 정렬·검증)

| API | endpoint | 제공 |
|-----|----------|------|
| OpenAlex | `api.openalex.org` | cited_by_count, source venue (무키) |
| Semantic Scholar | `api.semanticscholar.org/graph/v1` | 인용·influential citation |
| Crossref | `api.crossref.org` | DOI 메타·인용수 |

## 4. 사용 메모

- "검색 노출 ≠ 고품질" — 인용 전 §3 API로 venue + 인용수 cross-check.
- 저-IF OA(MDPI·Frontiers·Hindawi 등) 강제 배제 필요 시 검색 도구 `blocked_domains` 사용. 단 정당 venue 동반 배제 주의 — 무조건 배제 금지.
- 본 목록은 일반 reference (분야 무관 multidisciplinary + 광학·소자·바이오 high-IF 포함). 프로젝트 target subset·제출 순서는 `ProjectRule.md` 정의.
- 목록 갱신 시 본 파일만 수정 — rule 문서는 본 파일을 참조하므로 동기화 불필요.
