#Requires -Version 5.1
# ============================================
#   ELF Project Structure Generator
#   (Core + Modules)
#
#   Usage (Windows PowerShell 5.1):
#     powershell -ExecutionPolicy Bypass -File "elf-cli\ELF_generator.ps1"
#
#   Usage (PowerShell 7+):
#     pwsh -File "elf-cli\ELF_generator.ps1"
# ============================================

$ErrorActionPreference = 'Stop'

# Locate templates folder relative to this script
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$templatesDir = Join-Path $scriptDir 'templates'
if (-not (Test-Path $templatesDir)) {
    Write-Host "[Error] templates/ folder not found at: $templatesDir"
    exit 1
}

# Helper: Write content as UTF-8 without BOM
function Write-FileUTF8 {
    param([string]$Path, [string]$Content)
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    $fullPath = Join-Path (Get-Location).Path $Path
    [System.IO.File]::WriteAllText($fullPath, $Content, $utf8NoBom)
}

Write-Host ''
Write-Host '============================================'
Write-Host '  ELF Project Structure Generator'
Write-Host '  (Core + Modules)'
Write-Host '============================================'
Write-Host ''

# ================================================================
# 1. Project name input
# ================================================================
while ($true) {
    $projectName = Read-Host 'Enter project folder name'
    if ([string]::IsNullOrWhiteSpace($projectName)) {
        $projectName = 'New_ELF_Project'
        Write-Host "[Info] No name entered — using default: $projectName"
    }
    if (Test-Path $projectName) {
        Write-Host "[Error] '$projectName' already exists. Enter a different name."
    }
    else {
        break
    }
}

# ================================================================
# 2. Language selection
# ================================================================
$langNames = @(
    '',                  # 0: placeholder
    '한국어',            # 1
    'English',           # 2
    '日本語',            # 3
    '中文简体',          # 4
    '中文繁體',          # 5
    'Français',          # 6
    'Deutsch',           # 7
    'Español',           # 8
    'Italiano',          # 9
    'Português',         # 10
    'Русский',           # 11
    'العربية',           # 12
    'हिन्दी',            # 13
    'Türkçe',            # 14
    'Tiếng Việt',        # 15
    'ภาษาไทย',           # 16
    'Nederlands',        # 17
    'Polski',            # 18
    'Bahasa Indonesia'   # 19
)

Write-Host ''
Write-Host '  Select project language (AI agent response language):'
for ($i = 1; $i -le 19; $i++) {
    Write-Host ("   [{0,2}] {1}" -f $i, $langNames[$i])
}
Write-Host ''
$langChoice = Read-Host 'Enter number or type custom language [default: 1]'

if ([string]::IsNullOrWhiteSpace($langChoice)) {
    $projectLang = $langNames[1]
}
else {
    $langNum = 0
    if ([int]::TryParse($langChoice.Trim(), [ref]$langNum) -and $langNum -ge 1 -and $langNum -le 19) {
        $projectLang = $langNames[$langNum]
    }
    else {
        $projectLang = $langChoice.Trim()
    }
}

Write-Host ''
Write-Host "[1/7] Project root '$projectName' created (language: $projectLang)."

# ================================================================
# 3. Module preset selection
# ================================================================

# --- Core directories (always created) ---
$coreDirs = @(
    '0_Meta',
    '1_Concept/11_Literature',
    '1_Concept/12_Planning/Wiki',
    '1_Concept/12_Planning/Archive',
    '1_Concept/13_Ideas',
    '2_Log',
    '2_Log/Wiki',
    '2_Log/Archive',
    'templates'
)

# --- Module directories ---
$hwDirs = @(
    '3_HW/31_Component/Design',
    '3_HW/31_Component/Calibration',
    '3_HW/32_System',
    '3_HW/33_Elec'
)
$fabDirs = @(
    '4_Fab/41_Recipes',
    '4_Fab/42_Eval'
)
$swDirs = @(
    '5_SW/51_FW',
    '5_SW/52_DAQ',
    '5_SW/53_Libs'
)
$expDirs = @(
    '6_Exp/61_Sim/Scripts/Archive',
    '6_Exp/61_Sim/Data',
    '6_Exp/62_Empirical/Raw',
    '6_Exp/62_Empirical/Processed',
    '6_Exp/63_Analysis/Scripts/Archive',
    '6_Exp/64_Viz'
)
$paperDirs = @(
    '7_Paper/71_Figs/Raw',
    '7_Paper/71_Figs/Processed',
    '7_Paper/71_Figs/Final',
    '7_Paper/72_Drafts/Archive',
    '7_Paper/73_Presentations'
)

Write-Host ''
Write-Host '  Select module preset (Core 0~2 is always included):'
Write-Host '   [1] Full         — 3_HW + 4_Fab + 5_SW + 6_Exp + 7_Paper'
Write-Host '   [2] Experimental — 6_Exp + 7_Paper'
Write-Host '   [3] Software     — 5_SW + 7_Paper'
Write-Host '   [4] Minimal      — Core only (no modules)'
Write-Host '   [5] Custom       — Select individually'
Write-Host ''
$presetChoice = Read-Host 'Enter number [default: 1]'
if ([string]::IsNullOrWhiteSpace($presetChoice)) { $presetChoice = '1' }

$moduleDirs = @()
switch ($presetChoice.Trim()) {
    '1' {
        $moduleDirs = $hwDirs + $fabDirs + $swDirs + $expDirs + $paperDirs
        $presetName = 'Full'
    }
    '2' {
        $moduleDirs = $expDirs + $paperDirs
        $presetName = 'Experimental'
    }
    '3' {
        $moduleDirs = $swDirs + $paperDirs
        $presetName = 'Software'
    }
    '4' {
        $presetName = 'Minimal'
    }
    '5' {
        $presetName = 'Custom'
        Write-Host ''
        Write-Host '  Select modules to include:'
        $incHW = Read-Host '   3_HW  (Hardware)?      [y/N]'
        $incFab = Read-Host '   4_Fab (Fabrication)?   [y/N]'
        $incSW = Read-Host '   5_SW  (Software)?      [y/N]'
        $incExp = Read-Host '   6_Exp (Experiments)?   [y/N]'
        $incPaper = Read-Host '   7_Paper (Papers)?      [y/N]'
        if ($incHW -match '^[Yy]$') { $moduleDirs += $hwDirs }
        if ($incFab -match '^[Yy]$') { $moduleDirs += $fabDirs }
        if ($incSW -match '^[Yy]$') { $moduleDirs += $swDirs }
        if ($incExp -match '^[Yy]$') { $moduleDirs += $expDirs }
        if ($incPaper -match '^[Yy]$') { $moduleDirs += $paperDirs }
    }
    default {
        $moduleDirs = $hwDirs + $fabDirs + $swDirs + $expDirs + $paperDirs
        $presetName = 'Full'
    }
}

$dirs = $coreDirs + $moduleDirs

Write-Host "[2/7] Module preset: $presetName"

# ================================================================
# 4. Directory structure
# ================================================================
New-Item -ItemType Directory -Force -Path $projectName | Out-Null
Set-Location $projectName

foreach ($d in $dirs) {
    New-Item -ItemType Directory -Force -Path $d | Out-Null
}
Write-Host '[3/7] Directory structure created.'

# ================================================================
# 5. .gitkeep for empty folders (preserves structure on remote)
# ================================================================
foreach ($d in $dirs) {
    if ($d -eq '6_Exp/62_Empirical/Raw') {
        Write-FileUTF8 "$d/.gitignore" "*`n!.gitignore`n"
    }
    else {
        New-Item -ItemType File -Force -Path "$d/.gitkeep" | Out-Null
    }
}

New-Item -ItemType File -Force -Path '.gitattributes' | Out-Null
New-Item -ItemType File -Force -Path 'LICENSE' | Out-Null
Write-Host '[4/7] .gitkeep and empty files created.'

# ================================================================
# 6. Meta documents & config files
# ================================================================
$dateStr = Get-Date -Format 'yyyy-MM-dd'

# --- Copy template files ---
# Root files
$templateRoot = Join-Path $templatesDir 'root'
Copy-Item -Path "$templateRoot\.gitignore" -Destination '.gitignore' -Force

# Meta files
$templateMeta = Join-Path $templatesDir 'meta'
Copy-Item -Path "$templateMeta\LogConvention.md" -Destination '0_Meta\LogConvention.md' -Force
Copy-Item -Path "$templateMeta\AI_PARA_Framework.md" -Destination '0_Meta\AI_PARA_Framework.md' -Force
Copy-Item -Path "$templateMeta\AI_Sync.md" -Destination '0_Meta\AI_Sync.md' -Force
Copy-Item -Path "$templateMeta\highIFjournals.md" -Destination '0_Meta\highIFjournals.md' -Force

# EliRule is ELF-managed (project-agnostic) — copy as-is (lang은 .elf/config.json로 분리)
Copy-Item -Path "$templateMeta\EliRule.md" -Destination '0_Meta\EliRule.md' -Force

# --- Project Templates ---
$templateLog = Join-Path $templatesDir 'log'
Copy-Item -Path (Join-Path $templateLog 'sessionTemplate.md') -Destination 'templates\sessionTemplate.md' -Force -ErrorAction SilentlyContinue
Copy-Item -Path (Join-Path $templateLog 'trialTemplate.md') -Destination 'templates\trialTemplate.md' -Force -ErrorAction SilentlyContinue
Copy-Item -Path (Join-Path $templateLog 'Session_Registry.tsv') -Destination 'templates\Session_Registry.tsv' -Force -ErrorAction SilentlyContinue

# --- 0_Meta/ProjectRule.md (from template) ---
$projRuleContent = Get-Content (Join-Path $templateMeta 'ProjectRule.md') -Raw -Encoding UTF8
$projRuleContent = $projRuleContent.Replace('[프로젝트명]', $projectName).Replace('YYYY-MM-DD', $dateStr)
Write-FileUTF8 '0_Meta/ProjectRule.md' $projRuleContent

# --- README.md (from template) ---
$readmeContent = Get-Content -Path "$templateRoot\README.md" -Raw -Encoding UTF8
$readmeContent = $readmeContent.Replace('PLACEHOLDER_PROJECT_NAME', $projectName).Replace('PLACEHOLDER_DATE', $dateStr)
Write-FileUTF8 'README.md' $readmeContent

# --- S001_log.md (from template) ---
$logContent = Get-Content (Join-Path $templateLog 'sessionTemplate.md') -Raw -Encoding UTF8
$logContent = $logContent.Replace('S{NNN}', 'S001').Replace('YYYY-MM-DD', $dateStr)
Write-FileUTF8 '2_Log/S001_log.md' $logContent

# --- Session_Registry.tsv (from template) ---
$tsvContent = Get-Content (Join-Path $templateLog 'Session_Registry.tsv') -Raw -Encoding UTF8
$tsvContent = $tsvContent.Replace('YYYY-MM-DD', $dateStr)
Write-FileUTF8 '2_Log/Wiki/Session_Registry.tsv' $tsvContent

# --- .elf/ project config (lang·version를 콘텐츠 파일에서 분리) ---
$elfVersion = (Get-Content (Join-Path $scriptDir 'VERSION') -Raw).Trim()
New-Item -ItemType Directory -Path '.elf' -Force | Out-Null
$elfConfig = @"
{
  "name": "$projectName",
  "lang": "$projectLang",
  "created": "$dateStr"
}
"@
Write-FileUTF8 '.elf/config.json' $elfConfig
Write-FileUTF8 '.elf/version' "$elfVersion`n"

Write-Host '[5/7] Meta documents and config files created.'

# ================================================================
# 7. Git init & first commit (optional)
# ================================================================
Write-Host ''
if (Get-Command git -ErrorAction SilentlyContinue) {
    $gitChoice = Read-Host 'Initialize Git repository? [Y/n]'
    if ([string]::IsNullOrWhiteSpace($gitChoice) -or $gitChoice -match '^[Yy]') {
        try {
            git init
            git add .
            git commit -m "chore: Initialize ELF $elfVersion project structure"
            Write-Host '[6/7] Git initialized.'
        }
        catch {
            Write-Host "[6/7] Git init failed: $_"
        }
    }
    else {
        Write-Host '[6/7] Git initialization skipped.'
    }
}
else {
    Write-Host '[6/7] Git not found — skipping Git initialization.'
}

Set-Location ..

Write-Host ''
Write-Host '============================================'
Write-Host "  [$projectName] ELF $elfVersion project created!"
Write-Host "  Language: $projectLang"
Write-Host "  Modules:  $presetName"
Write-Host '============================================'
Write-Host '[7/7] Done!'
