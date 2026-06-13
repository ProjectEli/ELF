#!/usr/bin/env bash
# ============================================
#   ELF Project Structure Generator
#   (Core + Modules)
#
#   Usage:
#     chmod +x ELF_generator.sh && ./ELF_generator.sh
# ============================================

set -euo pipefail

# Locate templates folder relative to this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATES_DIR="$SCRIPT_DIR/templates"
if [[ ! -d "$TEMPLATES_DIR" ]]; then
    echo "[Error] templates/ folder not found at: $TEMPLATES_DIR"
    exit 1
fi

echo ''
echo '============================================'
echo '  ELF Project Structure Generator'
echo '  (Core + Modules)'
echo '============================================'
echo ''

# ================================================================
# 1. Project name input
# ================================================================
while true; do
    read -rp 'Enter project folder name: ' PROJECT_NAME
    if [[ -z "$PROJECT_NAME" ]]; then
        PROJECT_NAME='New_ELF_Project'
        echo "[Info] No name entered — using default: $PROJECT_NAME"
    fi
    if [[ -d "$PROJECT_NAME" ]]; then
        echo "[Error] '$PROJECT_NAME' already exists. Enter a different name."
    else
        break
    fi
done

# ================================================================
# 2. Language selection
# ================================================================
declare -a LANG_NAMES=(
    [1]='한국어'
    [2]='English'
    [3]='日本語'
    [4]='中文简体'
    [5]='中文繁體'
    [6]='Français'
    [7]='Deutsch'
    [8]='Español'
    [9]='Italiano'
    [10]='Português'
    [11]='Русский'
    [12]='العربية'
    [13]='हिन्दी'
    [14]='Türkçe'
    [15]='Tiếng Việt'
    [16]='ภาษาไทย'
    [17]='Nederlands'
    [18]='Polski'
    [19]='Bahasa Indonesia'
)

echo ''
echo '  Select project language (AI agent response language):'
for i in $(seq 1 19); do
    printf '   [%2d] %s\n' "$i" "${LANG_NAMES[$i]}"
done
echo ''
read -rp 'Enter number or type custom language [default: 1]: ' LANG_CHOICE
if [[ -z "$LANG_CHOICE" ]]; then
    PROJECT_LANG="${LANG_NAMES[1]}"
elif [[ "$LANG_CHOICE" =~ ^[0-9]+$ ]] && [[ "$LANG_CHOICE" -ge 1 && "$LANG_CHOICE" -le 19 ]] 2>/dev/null; then
    PROJECT_LANG="${LANG_NAMES[$LANG_CHOICE]}"
else
    PROJECT_LANG="$LANG_CHOICE"
fi

echo ''
echo "[1/7] Project root '$PROJECT_NAME' created (language: $PROJECT_LANG)."

# ================================================================
# 3. Module preset selection
# ================================================================

# --- Core directories (always created) ---
CORE_DIRS=(
    '0_Meta'
    '1_Concept/11_Literature'
    '1_Concept/12_Planning/Wiki'
    '1_Concept/12_Planning/Archive'
    '1_Concept/13_Ideas'
    '2_Log'
    '2_Log/Wiki'
    '2_Log/Archive'
    'templates'
)

# --- Module directories ---
HW_DIRS=(
    '3_HW/31_Component/Design'
    '3_HW/31_Component/Calibration'
    '3_HW/32_System'
    '3_HW/33_Elec'
)
FAB_DIRS=(
    '4_Fab/41_Recipes'
    '4_Fab/42_Eval'
)
SW_DIRS=(
    '5_SW/51_FW'
    '5_SW/52_DAQ'
    '5_SW/53_Libs'
)
EXP_DIRS=(
    '6_Exp/61_Sim/Scripts/Archive'
    '6_Exp/61_Sim/Data'
    '6_Exp/62_Empirical/Raw'
    '6_Exp/62_Empirical/Processed'
    '6_Exp/63_Analysis/Scripts/Archive'
    '6_Exp/64_Viz'
)
PAPER_DIRS=(
    '7_Paper/71_Figs/Raw'
    '7_Paper/71_Figs/Processed'
    '7_Paper/71_Figs/Final'
    '7_Paper/72_Drafts/Archive'
    '7_Paper/73_Presentations'
)

echo ''
echo '  Select module preset (Core 0~2 is always included):'
echo '   [1] Full         — 3_HW + 4_Fab + 5_SW + 6_Exp + 7_Paper'
echo '   [2] Experimental — 6_Exp + 7_Paper'
echo '   [3] Software     — 5_SW + 7_Paper'
echo '   [4] Minimal      — Core only (no modules)'
echo '   [5] Custom       — Select individually'
echo ''
read -rp 'Enter number [default: 1]: ' PRESET_CHOICE

MODULE_DIRS=()
case "${PRESET_CHOICE:-1}" in
    1)
        MODULE_DIRS+=("${HW_DIRS[@]}" "${FAB_DIRS[@]}" "${SW_DIRS[@]}" "${EXP_DIRS[@]}" "${PAPER_DIRS[@]}")
        PRESET_NAME="Full"
        ;;
    2)
        MODULE_DIRS+=("${EXP_DIRS[@]}" "${PAPER_DIRS[@]}")
        PRESET_NAME="Experimental"
        ;;
    3)
        MODULE_DIRS+=("${SW_DIRS[@]}" "${PAPER_DIRS[@]}")
        PRESET_NAME="Software"
        ;;
    4)
        PRESET_NAME="Minimal"
        ;;
    5)
        PRESET_NAME="Custom"
        echo ''
        echo '  Select modules to include:'
        read -rp '   3_HW  (Hardware)?      [y/N]: ' INC_HW
        read -rp '   4_Fab (Fabrication)?   [y/N]: ' INC_FAB
        read -rp '   5_SW  (Software)?      [y/N]: ' INC_SW
        read -rp '   6_Exp (Experiments)?   [y/N]: ' INC_EXP
        read -rp '   7_Paper (Papers)?      [y/N]: ' INC_PAPER
        [[ "${INC_HW:-n}" =~ ^[Yy]$ ]] && MODULE_DIRS+=("${HW_DIRS[@]}")
        [[ "${INC_FAB:-n}" =~ ^[Yy]$ ]] && MODULE_DIRS+=("${FAB_DIRS[@]}")
        [[ "${INC_SW:-n}" =~ ^[Yy]$ ]] && MODULE_DIRS+=("${SW_DIRS[@]}")
        [[ "${INC_EXP:-n}" =~ ^[Yy]$ ]] && MODULE_DIRS+=("${EXP_DIRS[@]}")
        [[ "${INC_PAPER:-n}" =~ ^[Yy]$ ]] && MODULE_DIRS+=("${PAPER_DIRS[@]}")
        ;;
    *)
        MODULE_DIRS+=("${HW_DIRS[@]}" "${FAB_DIRS[@]}" "${SW_DIRS[@]}" "${EXP_DIRS[@]}" "${PAPER_DIRS[@]}")
        PRESET_NAME="Full"
        ;;
esac

DIRS=("${CORE_DIRS[@]}" "${MODULE_DIRS[@]}")

echo "[2/7] Module preset: $PRESET_NAME"

# ================================================================
# 4. Directory structure
# ================================================================
mkdir -p "$PROJECT_NAME"
cd "$PROJECT_NAME"

for d in "${DIRS[@]}"; do
    mkdir -p "$d"
done
echo '[3/7] Directory structure created.'

# ================================================================
# 5. .gitkeep for empty folders (preserves structure on remote)
# ================================================================
for d in "${DIRS[@]}"; do
    if [[ "$d" == '6_Exp/62_Empirical/Raw' ]]; then
        printf '*\n!.gitignore\n' > "$d/.gitignore"
    else
        touch "$d/.gitkeep"
    fi
done

touch .gitattributes LICENSE
echo '[4/7] .gitkeep and empty files created.'

# ================================================================
# 6. Meta documents & config files
# ================================================================
DATE_STR=$(date +%Y-%m-%d)
# --- Copy template files ---
# Root files
cp "$TEMPLATES_DIR/root/.gitignore" .gitignore

# Meta files
cp "$TEMPLATES_DIR/meta/LogConvention.md" 0_Meta/LogConvention.md
cp "$TEMPLATES_DIR/meta/AI_PARA_Framework.md" 0_Meta/AI_PARA_Framework.md
cp "$TEMPLATES_DIR/meta/highIFjournals.md" 0_Meta/highIFjournals.md

# EliRule is ELF-managed (project-agnostic) — copy as-is (lang은 .elf/config.json로 분리)
cp "$TEMPLATES_DIR/meta/EliRule.md" 0_Meta/EliRule.md

# --- Project Templates ---
cp "$TEMPLATES_DIR/log/sessionTemplate.md" "templates/sessionTemplate.md" 2>/dev/null || true
cp "$TEMPLATES_DIR/log/trialTemplate.md" "templates/trialTemplate.md" 2>/dev/null || true
cp "$TEMPLATES_DIR/log/Session_Registry.tsv" "templates/Session_Registry.tsv" 2>/dev/null || true

# --- 0_Meta/ProjectRule.md (from template) ---
sed "s|\[프로젝트명\]|${PROJECT_NAME}|g; s|YYYY-MM-DD|${DATE_STR}|g" \
    "$TEMPLATES_DIR/meta/ProjectRule.md" > 0_Meta/ProjectRule.md

# --- README.md (from template) ---
sed "s|PLACEHOLDER_PROJECT_NAME|${PROJECT_NAME}|g; s|PLACEHOLDER_DATE|${DATE_STR}|g" \
    "$TEMPLATES_DIR/root/README.md" > README.md

# --- S001_log.md (from template) ---
sed "s|S{NNN}|S001|g; s|YYYY-MM-DD|${DATE_STR}|g" \
    "$TEMPLATES_DIR/log/sessionTemplate.md" > 2_Log/S001_log.md

# --- Session_Registry.tsv (from template) ---
sed "s|YYYY-MM-DD|${DATE_STR}|g" \
    "$TEMPLATES_DIR/log/Session_Registry.tsv" > 2_Log/Wiki/Session_Registry.tsv

# --- .elf/ project config (lang·version를 콘텐츠 파일에서 분리) ---
ELF_VERSION="$(tr -d '[:space:]' < "$SCRIPT_DIR/VERSION")"
mkdir -p .elf
cat > .elf/config.json <<EOF
{
  "name": "${PROJECT_NAME}",
  "lang": "${PROJECT_LANG}",
  "created": "${DATE_STR}"
}
EOF
printf '%s\n' "${ELF_VERSION}" > .elf/version
# manifest stamp (배포 버전 baseline) — 이게 있어야 `elf update`/`status`/`doctor`가 이 프로젝트를 인식
cp "$SCRIPT_DIR/manifest.json" .elf/manifest.json

echo '[5/7] Meta documents and config files created.'

# ================================================================
# 7. Git init & first commit (optional)
# ================================================================
echo ''
if command -v git &>/dev/null; then
    read -rp 'Initialize Git repository? [Y/n]: ' GIT_CHOICE
    if [[ -z "$GIT_CHOICE" || "$GIT_CHOICE" =~ ^[Yy]$ ]]; then
        git init
        git add .
        git commit -m "chore: Initialize ELF ${ELF_VERSION} project structure"
        echo '[6/7] Git initialized.'
    else
        echo '[6/7] Git initialization skipped.'
    fi
else
    echo '[6/7] Git not found — skipping Git initialization.'
fi

cd ..

echo ''
echo '============================================'
echo "  [$PROJECT_NAME] ELF ${ELF_VERSION} project created!"
echo "  Language: $PROJECT_LANG"
echo "  Modules:  $PRESET_NAME"
echo '============================================'
echo '[7/7] Done!'
