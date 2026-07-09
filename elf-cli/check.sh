#!/bin/sh
# ELF managed-mirror drift check (S005 t01, Phase 1).
# 정본 = elf-cli/templates/ (src, CLI 상대). 파생 = manifest dest (루트 상대 — v2.15+ .elf/managed/ 등). managed tier의 src/dest를 git hash-object로 비교(autocrlf/줄바꿈 안전).
# 불일치 시 비0 exit. seed/hybrid 제외. 미배포(dest 부재) skip.
# POSIX sh. LF. UTF-8 no BOM. 참조: _dev/1_Concept/12_Planning/Wiki/Git_Hook_Governance.md

CLI_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
MANIFEST="$CLI_DIR/manifest.json"

if [ ! -f "$MANIFEST" ]; then
    echo "[elf check] manifest 없음 ($MANIFEST) — skip"
    exit 0
fi

drift=$(grep '"tier": "managed"' "$MANIFEST" | while IFS= read -r line; do
    src=$(printf '%s' "$line" | sed -n 's/.*"src": *"\([^"]*\)".*/\1/p')
    dest=$(printf '%s' "$line" | sed -n 's/.*"dest": *"\([^"]*\)".*/\1/p')
    [ -z "$src" ] && continue
    srcp="$CLI_DIR/$src"
    destp="$ROOT/$dest"
    [ -f "$srcp" ] || continue
    [ -f "$destp" ] || continue
    sh=$(git hash-object "$srcp")
    dh=$(git hash-object "$destp")
    if [ "$sh" != "$dh" ]; then
        echo "$dest  !=  $src"
    fi
done)

if [ -n "$drift" ]; then
    echo "--------------------------------------------------------"
    echo "[Blocked] ELF managed 파일이 정본(elf-cli/templates/)과 불일치:"
    printf '%s\n' "$drift" | sed 's/^/  /'
    echo "  정본 = elf-cli/templates/ . 편집은 elf-cli/templates/ 에서 하거나 sync 후 commit."
    echo "  manifest 갱신 필요 시 elf-cli/manifest.json sha256 재산출."
    echo "  우회(비권장): git commit --no-verify"
    echo "--------------------------------------------------------"
    exit 1
fi

exit 0
