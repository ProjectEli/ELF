#!/usr/bin/env bash
# ============================================
#   ELF v2 Project Structure Generator
#   (0~6 Hierarchy + PARA Framework)
#
#   Usage:
#     chmod +x ELF_generator.sh && ./ELF_generator.sh
# ============================================

set -euo pipefail

# Locate templates folder relative to this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATES_DIR="$SCRIPT_DIR/../templates"
if [[ ! -d "$TEMPLATES_DIR" ]]; then
    echo "[Error] templates/ folder not found at: $TEMPLATES_DIR"
    exit 1
fi

echo ''
echo '============================================'
echo '  ELF v2 Project Structure Generator'
echo '  (0~6 Hierarchy + PARA Framework)'
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
echo "[1/6] Project root '$PROJECT_NAME' created (language: $PROJECT_LANG)."

# ================================================================
# 3. Directory structure (ELF v2: 0~6)
# ================================================================
DIRS=(
    '0_Meta'
    '0_Meta/scripts'
    '1_Concept/11_Ideas'
    '1_Concept/12_Literature'
    '1_Concept/13_Planning/2_Wiki'
    '1_Concept/13_Planning/9_Archive'
    '2_HW/21_Component/Design'
    '2_HW/21_Component/Calibration'
    '2_HW/22_System'
    '2_HW/23_Elec'
    '3_Fab/31_Recipes'
    '3_Fab/32_Eval'
    '4_SW/41_FW'
    '4_SW/42_DAQ'
    '4_SW/43_Libs'
    '5_Exp/51_Sim/Scripts/9_Archive'
    '5_Exp/51_Sim/Data'
    '5_Exp/52_Empirical/Raw'
    '5_Exp/52_Empirical/Processed'
    '5_Exp/53_Analysis/Scripts/9_Archive'
    '5_Exp/53_Analysis/Logs/2_Wiki'
    '5_Exp/53_Analysis/Logs/9_Archive'
    '5_Exp/54_Viz'
    '6_Paper/61_Figs/Raw'
    '6_Paper/61_Figs/Processed'
    '6_Paper/61_Figs/Final'
    '6_Paper/62_Drafts/9_Archive'
    '6_Paper/63_Presentations'
    'templates'
)

mkdir -p "$PROJECT_NAME"
cd "$PROJECT_NAME"

for d in "${DIRS[@]}"; do
    mkdir -p "$d"
done
echo '[2/6] Directory structure created (0_Meta ~ 6_Paper).'

# ================================================================
# 4. .gitkeep for empty folders (preserves structure on remote)
# ================================================================
for d in "${DIRS[@]}"; do
    if [[ "$d" == '5_Exp/52_Empirical/Raw' ]]; then
        printf '*\n!.gitignore\n' > "$d/.gitignore"
    else
        touch "$d/.gitkeep"
    fi
done

touch .gitattributes LICENSE
echo '[3/6] .gitkeep and empty files created.'

# ================================================================
# 5. Meta documents & config files
# ================================================================
DATE_STR=$(date +%Y-%m-%d)
# --- Copy template files ---
# Root files
cp "$TEMPLATES_DIR/root/.gitignore" .gitignore

# Meta files
cp "$TEMPLATES_DIR/meta/LogConvention.md" 0_Meta/LogConvention.md
cp "$TEMPLATES_DIR/meta/AI_PARA_Framework.md" 0_Meta/AI_PARA_Framework.md
cp "$TEMPLATES_DIR/meta/AI_Sync.md" 0_Meta/AI_Sync.md

sed "s|PLACEHOLDER_PROJECT_LANG|${PROJECT_LANG}|g" \
    "$TEMPLATES_DIR/meta/EliRule.md" > 0_Meta/EliRule.md

# --- 0_Meta/scripts/ ---
if [[ -d "$TEMPLATES_DIR/scripts" ]]; then
    cp "$TEMPLATES_DIR/scripts/"* "0_Meta/scripts/" 2>/dev/null || true
    chmod +x 0_Meta/scripts/*.sh 2>/dev/null || true
fi

# --- 0_Meta/ProjectRule.md (from template) ---
sed "s|\[프로젝트명\]|${PROJECT_NAME}|g; s|YYYY-MM-DD|${DATE_STR}|g" \
    "$TEMPLATES_DIR/ProjectRule.md" > 0_Meta/ProjectRule.md

# --- README.md (from template) ---
sed "s|PLACEHOLDER_PROJECT_NAME|${PROJECT_NAME}|g; s|PLACEHOLDER_DATE|${DATE_STR}|g" \
    "$TEMPLATES_DIR/root/README.md" > README.md

# --- S001_log.md (from template) ---
sed "s|S{NNN}|S001|g; s|YYYY-MM-DD|${DATE_STR}|g" \
    "$TEMPLATES_DIR/sessionTemplate.md" > 5_Exp/53_Analysis/Logs/S001_log.md

# --- Session_Registry.tsv ---
printf 'Session\tDate\tTitle\tStatus\tKey Finding\tArchive Path\r\n' > 5_Exp/53_Analysis/Logs/2_Wiki/Session_Registry.tsv
printf 'S001\t%s\t[세션 제목]\t★ 활성\t-\t-\r\n' "$DATE_STR" >> 5_Exp/53_Analysis/Logs/2_Wiki/Session_Registry.tsv

echo '[4/6] Meta documents and config files created.'

# ================================================================
# 6. Git init & first commit (optional)
# ================================================================
echo ''
if command -v git &>/dev/null; then
    read -rp 'Initialize Git repository? [Y/n]: ' GIT_CHOICE
    if [[ -z "$GIT_CHOICE" || "$GIT_CHOICE" =~ ^[Yy]$ ]]; then
        git init
        git add .
        git commit -m 'chore: Initialize ELF v2 project structure'
        echo '[5/6] Git initialized.'
    else
        echo '[5/6] Git initialization skipped.'
    fi
else
    echo '[5/6] Git not found — skipping Git initialization.'
fi

cd ..

echo ''
echo '============================================'
echo "  [$PROJECT_NAME] ELF v2 project created!"
echo "  Language: $PROJECT_LANG"
echo '============================================'
echo '[6/6] Done!'
