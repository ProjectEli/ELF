"""`src/earlier_files.rs` 생성 — `.elf/` 없는 ELF 2.0~2.4.0 프로젝트에 ELF가 넣은 파일의 지문.
`elf update`가 그 파일이 당시 원본 그대로인지(사용자가 고치지 않았는지) 판정하는 데 씀(v3.0.2, S046).

출처 = 이 저장소의 git tag:
- v2.0~v2.3: `0_Meta/ELF_generator.ps1`이 복사한 틀(`templates/…`). EliRule은 `PLACEHOLDER_PROJECT_LANG`만 언어로 바꿔 씀
  → 앞부분과 뒷부분 지문(Around). 나머지는 그대로 복사 → 전체 지문(Exact).
- v2.4.0: CLI `init`이 manifest(`elf-cli/manifest.json`)대로 그대로 씀(managed, hybrid). 이 버전 `init`은 도중에 멈춰
  `.elf/`를 만들지 못했으므로 지문이 필요함(2.4.1부터는 프로젝트 manifest의 sha로 판정).

지문 = sha256(LF 정규화 바이트, `hash::sha256_lf`와 같음). Around는 BOM을 뗀 LF 정규화 텍스트 기준.
사용(저장소 어디서나): python -I elf-cli/cli/tools/earlier_files.py
"""
import hashlib
import json
import os
import subprocess
import sys

sys.stdout.reconfigure(encoding="utf-8")
REPO = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
OUT = os.path.join(REPO, "elf-cli", "cli", "src", "earlier_files.rs")
GEN = ["v2.0", "v2.1", "v2.2", "v2.3"]
PLACEHOLDER = b"PLACEHOLDER_PROJECT_LANG"


def show(tag, path):
    p = subprocess.run(["git", "-C", REPO, "show", f"{tag}:{path}"], capture_output=True)
    return p.stdout if p.returncode == 0 else None


def ls(tag, prefix):
    p = subprocess.run(["git", "-C", REPO, "ls-tree", "-r", "--name-only", tag, prefix], capture_output=True, text=True)
    return [l for l in p.stdout.splitlines() if l]


def lf(b):
    return b.replace(b"\r\n", b"\n")


def sha(b):
    return hashlib.sha256(b).hexdigest()


entries = []  # (dest, tier, since, kind, data)
seen = set()


def add(dest, tier, since, kind, data):
    key = (dest, kind, data)
    if key in seen:
        return
    seen.add(key)
    entries.append((dest, tier, since, kind, data))


for tag in GEN:
    log_dir = "templates/log/" if show(tag, "templates/log/sessionTemplate.md") is not None else "templates/"
    pairs = [
        (".gitignore", "hybrid", "templates/root/.gitignore"),
        ("0_Meta/LogConvention.md", "managed", "templates/meta/LogConvention.md"),
        ("0_Meta/AI_PARA_Framework.md", "managed", "templates/meta/AI_PARA_Framework.md"),
        ("templates/sessionTemplate.md", "managed", log_dir + "sessionTemplate.md"),
        ("templates/trialTemplate.md", "managed", log_dir + "trialTemplate.md"),
        ("templates/Session_Registry.tsv", "managed", "templates/log/Session_Registry.tsv"),
    ]
    for f in ls(tag, "templates/scripts/"):
        pairs.append(("0_Meta/scripts/" + f.rsplit("/", 1)[1], "managed", f))
    for dest, tier, src in pairs:
        b = show(tag, src)
        if b is not None:
            add(dest, tier, tag, "Exact", sha(lf(b)))
    eli = show(tag, "templates/meta/EliRule.md")
    if eli is not None:
        t = lf(eli)
        if t.startswith(b"\xef\xbb\xbf"):
            t = t[3:]
        parts = t.split(PLACEHOLDER)
        if len(parts) == 2:
            add("0_Meta/EliRule.md", "managed", tag, "Around", (len(parts[0]), sha(parts[0]), len(parts[1]), sha(parts[1])))
        else:
            add("0_Meta/EliRule.md", "managed", tag, "Exact", sha(t))

m = json.loads(show("v2.4.0", "elf-cli/manifest.json"))
for e in m["files"]:
    if e["tier"] in ("managed", "hybrid"):
        b = show("v2.4.0", "elf-cli/" + e["src"])
        if b is not None:
            add(e["dest"], e["tier"], "v2.4.0", "Exact", sha(lf(b)))

lines = [
    "//! 생성 파일 — `tools/earlier_files.py`가 git tag(v2.0~v2.3 generator 틀, v2.4.0 CLI manifest)에서 만듦. 손으로 고치지 않음.",
    "//! `.elf/` 없는 ELF 2.0~2.4.0 프로젝트의 ELF 파일이 당시 원본 그대로인지 판정하는 지문(v3.0.2, S046).",
    "",
    "/// 지문 — LF 정규화 바이트의 sha256. `Around` = 언어 한 줄만 바꿔 쓴 파일(EliRule)의 앞부분과 뒷부분(BOM 제외).",
    "pub(crate) enum Print {",
    "    Exact(&'static str),",
    "    Around { head_len: usize, head: &'static str, tail_len: usize, tail: &'static str },",
    "}",
    "",
    "/// 알려진 원본 하나: 프로젝트 안 위치, 관리 방식(`managed`, `hybrid`), 처음 나온 버전, 지문.",
    "pub(crate) struct Known {",
    "    pub dest: &'static str,",
    "    pub tier: &'static str,",
    "    pub since: &'static str,",
    "    pub print: Print,",
    "}",
    "",
    "pub(crate) const KNOWN: &[Known] = &[",
]
for dest, tier, since, kind, data in entries:
    if kind == "Exact":
        p = f'Print::Exact("{data}")'
    else:
        hl, h, tl, tt = data
        p = f'Print::Around {{ head_len: {hl}, head: "{h}", tail_len: {tl}, tail: "{tt}" }}'
    lines.append(f'    Known {{ dest: "{dest}", tier: "{tier}", since: "{since}", print: {p} }},')
lines.append("];")
open(OUT, "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
print(f"{len(entries)} entries -> {OUT}")
