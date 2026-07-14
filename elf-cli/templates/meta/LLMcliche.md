# LLMcliche: LLM 상투 표현 배제 참고자료 (Reference)

> **위상**: 본 문서는 `EliRule.md §3 rule 12`(LLM Cliché Ban)의 *원칙* 적용을 돕는 **비망라·비구속 참고자료(reference)**다. **고정 whitelist/blacklist 아님** — 미수록 어휘도 상투적이면 배제 대상이며, 수록 어휘도 정확한 기술적 의미로 쓰면 허용된다. 규범은 EliRule의 원칙이고 본 목록은 예시일 뿐임. 모델은 자주 갱신되고 출력이 크게 중첩되므로 forensic fingerprint로 쓰지 말 것.

배제 기준(요약): 막연한 filler·과용 register·voiceless 상투구는 배제하고 **구체·능동·직접 서술**로 대체. 영어 출력 한정(한국어 dry 로그 무관; 영어 어휘·문구 혼용 시 적용).

---

## 1. 어휘 상투 (카테고리별, 비망라)

| 카테고리 | 예시 어휘 |
|---|---|
| 상투 동사 | represent · serve/stand/act as · delve · navigate · leverage · harness · showcase · underscore · foster · enhance · alleviate · unlock · elevate · streamline · spearhead · embark · garner · bolster · revolutionize · shed light on · (막연한) yield · constitute("X constitutes Y" 막연 계사 — is로) · intensify(막연 심화 과장 — increase로; 물리 강도 정량은 §4) · tie (A to B)(결속 상투 — link로) |
| 막연 형용사 | crucial · vital · pivotal · essential · seamless · intricate · profound · paramount · nuanced · multifaceted · holistic · vibrant · innovative · cutting-edge · meticulous · (막연한) robust · (막연한) comprehensive · (막연한) trivial |
| 상투 명사 | tapestry · landscape · realm · paradigm · testament · cornerstone · beacon · myriad · plethora · synergy · ecosystem · verdict(평가·검사 결과를 사법 register로 격상 — result/pass-fail로) · readout(계측 결과의 막연 명사화 — measurement/signal로; 하드웨어 판독 맥락은 §4) · X family(방법론의 비학술 은유 분류 — methods 등 구체 분류어로; 정의된 분류 용법은 §4) · provenance(출처의 과장 명사화 — source·출처로; 데이터 계보 표준 용법은 §4) · dossier(정리·취합의 비학술 격상 — record/summary로) · caveat(한계·단서의 상투 헤지 명사 — limitation·유의점으로) |
| 과용 연결어 | Moreover · Furthermore · Consequently · Additionally · 문두 연속 However · Notably · Importantly · Indeed · Nonetheless |
| 상투 구문 | "it is worth/important noting that" · "plays a crucial/pivotal role" · "pave the way" · "at the forefront of" · "in the realm of" · "a wide range/array of" · "rich tapestry" · "ever/rapidly evolving" · "revolutionize the way" · "in parallel (with)"(막연 병렬 서술 연결구) |
| 강조 부사 | uniquely · fundamentally · primarily · meticulously · crystallize |

- metaphor성 치환(렌즈/lens · navigate · deep dive · journey · shed light on · crystallize)은 `EliRule §3 rule 3`(비유 금지) 소관 — 본 항목은 비은유 어휘·register 중심.

## 2. 어시스턴트 register · 대화 상투 (cross-model, 비망라)

채팅·코딩 어시스턴트(ChatGPT·Claude·Gemini 등) 응답에 공통으로 나타나는 register 상투. 로그·문서·커밋 메시지에 침투 주의.

| 유형 | 예시 |
|---|---|
| 시작 인사·과잉 동의 | "Certainly!" · "Absolutely!" · "Sure!" · "Great question" · "You're absolutely right" |
| 자기언급 | "I'd be happy to help" · "Let me explain" · "Let me break this down" · "I hope this helps" · "feel free to" |
| 헤지·단서 과다 | "It's worth noting that" · "It's important to note/remember" · "Generally speaking" · "In many cases" · "While this may vary" |
| 정형 구문 | "not only … but also" · "It's not X, it's Y"(동일 글 내 반복) · 마무리 "In conclusion / In summary" |
| 구조 tic | rule-of-three 강박 나열 · "**Bold 용어**: 설명문" 한 줄 항목 반복 · 불릿 과다 |
| 문장부호 | em-dash(—) 단락당 다회 과용(쉼표·괄호로 충분한 자리) |

## 3. 모델별 경향 (model-leaning, 비결정적·중첩)

> 주의: 출력이 크게 중첩되고 모델 갱신마다 변함 — *경향*이지 절대 분류 아님.

| 모델 | 보고된 경향 | 근거 tier |
|---|---|---|
| GPT / ChatGPT | excess vocabulary 급증 어휘(delve · underscore · meticulous · boast · intricate · tapestry); 격식·임상적 톤 | 학술(Kobak · FSU) |
| Claude / Claude Code | em-dash 과용; qualifier·hedge 과다; 자기언급("I'd be happy to help" · "Let me"); "**Bold**: 설명" 리스트; 곁가지 윤리·일반화 | 커뮤니티 관찰(비학술) |
| Gemini | 대화적·접근적 평이체; 불릿 과다; 장문서에서 요청 voice가 정보톤으로 drift | 보도(Scientific American) |

## 4. 예외 — 정확한 기술적 사용 (허용, 비망라)

배제군 어휘라도 **정확한 기술적 denotation**으로 쓰면 허용. 예시:

| 어휘 | 허용 맥락 |
|---|---|
| robust / robustness | 통계·제어 강건성 (robust estimator 등) |
| significance / significant | 통계적 유의성 (p-value) — "significant improvement"식 막연 강조는 배제 |
| comprehensive | 실제 전수(全數)·망라 의미일 때 |
| vital | vital sign(의학) 등 표준 용어 |
| essential | essential amino acid 등 표준 용어 |
| yield | yield strength · 반응 수율 · yield curve 등 정량 |
| trivial | 수학·CS의 trivial/non-trivial solution 등 정의어 |
| readout | 계측 하드웨어의 판독 회로·신호 경로 (detector readout 등) — 결과·신호 일반 지칭은 measurement/signal |
| family | 수학·통계의 family of curves/distributions, 네트워크 protocol family 등 정의된 분류 단위 — "the X family of methods"식 막연 은유 분류는 배제 |
| intensify | 물리량 강도(intensity)의 정량 증가 — 막연한 심화·악화 표현은 배제 |
| provenance | 데이터 계보 추적의 표준 용법(data provenance·W3C PROV·재현성 기록) — 일반적 "출처" 지칭은 source·출처 |

- **도메인 동음어**(예: energy/fitness `landscape`, 실험심리 `paradigm`, `realm`)는 프로젝트마다 다름 → **`ProjectRule.md`에 carve-out 선언**. 본 문서·EliRule은 일반 원칙만 유지(프로젝트별 whitelist 비축적).

## 5. 전후 예시 (Before / After, 패턴 예시 — 도메인 무관)

| Before (LLM 상투) | After (구체·능동·직접) |
|---|---|
| X **represents** a major challenge | X **is** a major challenge |
| The method **provides crucial insights** | The method **measures** [구체 대상·수치] |
| The signal is **uniquely governed** by geometry | The signal **is governed** by geometry |
| **By leveraging** the calibration | **Using** the calibration |
| The result is, **importantly**, **not trivial** | The effect **is 3× the baseline** (헤지 삭제·크기 명시) |
| The checker returns its **verdict** | The checker **returns a result / passes or fails** |
| X **constitutes** a limitation that **intensifies** over time | X **is** a limitation that **increases** over time |
| **A caveat**: the dataset's **provenance** is unclear | **A limitation**: the dataset's **source** is unclear |

---

## 참조
- 규범(원칙): `.elf/managed/EliRule.md` §3 rule 12 (LLM Cliché Ban) — 본 문서는 그 적용 예시.
- 은유 배제: `EliRule.md` §3 rule 3.
- 프로젝트별 기술 동음어 예외: `ProjectRule.md`.
- 프로젝트 어휘 추가·제외·재정의(overlay): `0_Meta/LLMcliche.project.md` — base ⊕ overlay, 제외는 사유 필수 (EliRule §2.7).

## 근거 (Sources)
- Kobak et al., "Delving into LLM-assisted writing in biomedical publications through excess vocabulary," arXiv:2406.07016 (Science Advances, 2025) — PubMed 1,400만 초록, excess word delve·underscore·meticulous·boast 등.
- FSU, "Why Does ChatGPT 'Delve' So Much?" (Proc. COLING 2025) — 과대표상 원인.
- Scientific American (2025), "ChatGPT and Gemini AI Have Uniquely Different Writing Styles."
- 주의: 모델별 세부 tell 일부는 커뮤니티 관찰(비학술 tier)이며, 본 목록은 register 기반·비망라. 신규 모델·갱신 시 변동.
