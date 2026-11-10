"""`tests/fixtures/earlier/<버전>.tar.xz` 생성 — 실제 2.x 릴리즈로 만든 프로젝트(시험 자료, v3.0.2 S046).

- generator 시대(v2.0, v2.1, v2.3): 그 tag의 `0_Meta/ELF_generator.ps1`을 입력과 함께 실행(v2.0은 pwsh — Windows PowerShell 5.1은
  BOM 없는 UTF-8 스크립트를 한글 코드페이지로 읽어 파싱 실패).
- CLI 시대(v2.4.0, v2.4.5, v2.14.0, v2.20.0, v2.27.4): GitHub 릴리즈 실행 파일로 `init proj`와 `session new`(있으면).
- 덧붙임: v2.0, v2.3 = 보관 폴더(`9_Archive`)의 닫은 세션과 TSV 행, v2.3 = 사용자가 고친 EliRule과 `.gitignore`,
  v2.14.0 = 사용자가 고친 LogConvention.
압축 = ustar tar + xz(프로젝트 루트 기준 경로, `.git` 제외). Windows 전용(powershell, 릴리즈 zip).
사용(저장소 어디서나): python -I elf-cli/cli/tools/earlier_fixtures.py
"""
import io
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile

sys.stdout.reconfigure(encoding="utf-8")
REPO = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
OUT = os.path.join(REPO, "elf-cli", "cli", "tests", "fixtures", "earlier")
ASSET = "elf-cli-x86_64-pc-windows-msvc.zip"
GEN = ["v2.0", "v2.1", "v2.3"]
CLI = ["v2.4.0", "v2.4.5", "v2.14.0", "v2.20.0", "v2.27.4"]


def run(cmd, cwd, stdin=""):
    env = dict(os.environ)
    env["ELF_DISPATCH"] = "off"
    p = subprocess.run(cmd, cwd=cwd, env=env, input=stdin, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return p.returncode, p.stdout + p.stderr


def release_exe(tag, work):
    url = f"https://github.com/ProjectEli/ELF/releases/download/{tag}/{ASSET}"
    data = urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "elf-fixtures"}), timeout=60).read()
    with zipfile.ZipFile(io.BytesIO(data)) as z:
        name = next(n for n in z.namelist() if n.endswith("elf.exe"))
        exe = os.path.join(work, f"elf-{tag}.exe")
        open(exe, "wb").write(z.read(name))
    return exe


def generator(tag, work):
    src = os.path.join(work, f"src-{tag}")
    os.makedirs(src)
    arch = subprocess.run(["git", "-C", REPO, "archive", tag], capture_output=True, check=True).stdout
    subprocess.run(["tar", "-x", "-C", src], input=arch, check=True)
    base = os.path.join(work, tag)
    os.makedirs(base)
    shell = "pwsh" if tag == "v2.0" else "powershell"
    answers = "proj\n1\n" + ("1\n" if tag == "v2.3" else "") + "n\n"
    rc, out = run([shell, "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", os.path.join(src, "0_Meta", "ELF_generator.ps1")], base, answers)
    assert rc == 0, out
    return os.path.join(base, "proj")


def cli(tag, work):
    exe = release_exe(tag, work)
    base = os.path.join(work, tag)
    os.makedirs(base)
    run([exe, "init", "proj"], base, "\n")  # v2.4.0 = init이 도중에 멈춤(그대로 시험 자료)
    proj = os.path.join(base, "proj")
    run([exe, "session", "new", "probe session"], proj, "\n")
    return proj


def append(path, text):
    with open(path, "a", encoding="utf-8", newline="") as f:
        f.write(text)


def archived_session(proj, log_dir):
    """보관 폴더의 닫은 세션 S002와 TSV 행(열 수는 그 TSV 머리줄을 따름)."""
    os.makedirs(os.path.join(proj, log_dir, "9_Archive"), exist_ok=True)
    body = (
        "# S002: 닫은 세션\n\n> **Created**: 2026-04-06\n> **Modified**: 2026-04-07\n> **Status**: Complete\n"
        "> **목표**: 보관 폴더 이관 확인\n> **Handoff**: -\n\n---\n\n## t01: 측정\n\n### 목표 (Goal)\n- 측정\n\n"
        "### 관찰 (Observation)\n- [S001](../S001_log.md)과 같은 결과\n"
    )
    open(os.path.join(proj, log_dir, "9_Archive", "S002_log.md"), "w", encoding="utf-8", newline="\n").write(body)
    wiki = next(w for w in ("2_Wiki", "1_Wiki") if os.path.isdir(os.path.join(proj, log_dir, w)))
    tsv = os.path.join(proj, log_dir, wiki, "Session_Registry.tsv")
    cols = open(tsv, encoding="utf-8").read().splitlines()[0].split("\t")
    row = ["S002", "2026-04-06", "닫은 세션", "Complete", "측정값 확인", "9_Archive/S002_log.md"][: len(cols)]
    row += ["-"] * (len(cols) - len(row))
    text = open(tsv, encoding="utf-8").read()
    append(tsv, ("" if text.endswith("\n") else "\n") + "\t".join(row) + "\n")


def pack(proj, tag):
    os.makedirs(OUT, exist_ok=True)
    dest = os.path.join(OUT, f"{tag}.tar.xz")
    with tarfile.open(dest, "w:xz", format=tarfile.USTAR_FORMAT) as t:
        for d, dirs, files in os.walk(proj):
            dirs[:] = sorted(x for x in dirs if x != ".git")
            for f in sorted(files):
                full = os.path.join(d, f)
                rel = os.path.relpath(full, proj).replace("\\", "/")
                info = t.gettarinfo(full, rel)
                info.uid = info.gid = 0
                info.uname = info.gname = ""
                info.mtime = 0
                with open(full, "rb") as fh:
                    t.addfile(info, fh)
    print(f"{tag}: {os.path.getsize(dest)} bytes -> {dest}")


with tempfile.TemporaryDirectory() as work:
    for tag in GEN + CLI:
        proj = generator(tag, work) if tag in GEN else cli(tag, work)
        if tag == "v2.0":
            archived_session(proj, "5_Exp/53_Analysis/Logs")
        if tag == "v2.3":
            archived_session(proj, "2_Log")
            append(os.path.join(proj, "0_Meta", "EliRule.md"), "\n- 사용자 추가 규칙: 측정 단위는 SI\n")
            append(os.path.join(proj, ".gitignore"), "\nmy_local/\n")
        if tag == "v2.14.0":
            append(os.path.join(proj, "0_Meta", "LogConvention.md"), "\n- 사용자 추가 규칙: trial 제목에 날짜\n")
        pack(proj, tag)
    shutil.rmtree(work, ignore_errors=True)
