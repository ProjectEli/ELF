#Requires -Version 5.1
# ============================================
#   ELF v2.1 Project Structure Generator
#   (0~7 Hierarchy + PARA Framework)
#
#   Usage (Windows PowerShell 5.1):
#     powershell -ExecutionPolicy Bypass -File "0_Meta\ELF_generator.ps1"
#
#   Usage (PowerShell 7+):
#     pwsh -File "0_Meta\ELF_generator.ps1"
# ============================================

$ErrorActionPreference = 'Stop'

# Locate templates folder relative to this script
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$templatesDir = Join-Path $scriptDir '..\templates'
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
Write-Host '  ELF v2.1 Project Structure Generator'
Write-Host '  (0~7 Hierarchy + PARA Framework)'
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
    } else {
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
} else {
    $langNum = 0
    if ([int]::TryParse($langChoice.Trim(), [ref]$langNum) -and $langNum -ge 1 -and $langNum -le 19) {
        $projectLang = $langNames[$langNum]
    } else {
        $projectLang = $langChoice.Trim()
    }
}

Write-Host ''
Write-Host "[1/6] Project root '$projectName' created (language: $projectLang)."

# ================================================================
# 3. Directory structure (ELF v2.1: 0~7)
# ================================================================
$dirs = @(
    '0_Meta',
    '0_Meta/scripts',
    '1_Concept/11_Ideas',
    '1_Concept/12_Literature',
    '1_Concept/13_Planning/2_Wiki',
    '1_Concept/13_Planning/9_Archive',
    '2_HW/21_Component/Design',
    '2_HW/21_Component/Calibration',
    '2_HW/22_System',
    '2_HW/23_Elec',
    '3_Fab/31_Recipes',
    '3_Fab/32_Eval',
    '4_SW/41_FW',
    '4_SW/42_DAQ',
    '4_SW/43_Libs',
    '5_Exp/51_Sim/Scripts/9_Archive',
    '5_Exp/51_Sim/Data',
    '5_Exp/52_Empirical/Raw',
    '5_Exp/52_Empirical/Processed',
    '5_Exp/53_Analysis/Scripts/9_Archive',
    '5_Exp/54_Viz',
    '7_Log',
    '7_Log/2_Wiki',
    '7_Log/9_Archive',
    '6_Paper/61_Figs/Raw',
    '6_Paper/61_Figs/Processed',
    '6_Paper/61_Figs/Final',
    '6_Paper/62_Drafts/9_Archive',
    '6_Paper/63_Presentations',
    'templates'
)

New-Item -ItemType Directory -Force -Path $projectName | Out-Null
Set-Location $projectName

foreach ($d in $dirs) {
    New-Item -ItemType Directory -Force -Path $d | Out-Null
}
Write-Host '[2/6] Directory structure created (0_Meta ~ 7_Log).'

# ================================================================
# 4. .gitkeep for empty folders (preserves structure on remote)
# ================================================================
foreach ($d in $dirs) {
    if ($d -eq '5_Exp/52_Empirical/Raw') {
        Write-FileUTF8 "$d/.gitignore" "*`n!.gitignore`n"
    } else {
        New-Item -ItemType File -Force -Path "$d/.gitkeep" | Out-Null
    }
}

New-Item -ItemType File -Force -Path '.gitattributes' | Out-Null
New-Item -ItemType File -Force -Path 'LICENSE' | Out-Null
Write-Host '[3/6] .gitkeep and empty files created.'

# ================================================================
# 5. Meta documents & config files
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

$eliRuleContent = Get-Content -Path "$templateMeta\EliRule.md" -Raw -Encoding UTF8
$eliRuleContent = $eliRuleContent.Replace('PLACEHOLDER_PROJECT_LANG', $projectLang)
Write-FileUTF8 '0_Meta\EliRule.md' $eliRuleContent

# --- 0_Meta/scripts/ ---
$templateScriptsDir = Join-Path $templatesDir 'scripts'
if (Test-Path $templateScriptsDir) {
    Copy-Item -Path "$templateScriptsDir\*" -Destination '0_Meta/scripts' -Force -ErrorAction SilentlyContinue
}

# --- Project Templates ---
Copy-Item -Path (Join-Path $templatesDir 'sessionTemplate.md') -Destination 'templates\sessionTemplate.md' -Force -ErrorAction SilentlyContinue
Copy-Item -Path (Join-Path $templatesDir 'trialTemplate.md') -Destination 'templates\trialTemplate.md' -Force -ErrorAction SilentlyContinue

# --- 0_Meta/ProjectRule.md (from template) ---
$projRuleContent = Get-Content (Join-Path $templatesDir 'ProjectRule.md') -Raw -Encoding UTF8
$projRuleContent = $projRuleContent.Replace('[프로젝트명]', $projectName).Replace('YYYY-MM-DD', $dateStr)
Write-FileUTF8 '0_Meta/ProjectRule.md' $projRuleContent

# --- README.md (from template) ---
$readmeContent = Get-Content -Path "$templateRoot\README.md" -Raw -Encoding UTF8
$readmeContent = $readmeContent.Replace('PLACEHOLDER_PROJECT_NAME', $projectName).Replace('PLACEHOLDER_DATE', $dateStr)
Write-FileUTF8 'README.md' $readmeContent

# --- S001_log.md (from template) ---
$logContent = Get-Content (Join-Path $templatesDir 'sessionTemplate.md') -Raw -Encoding UTF8
$logContent = $logContent.Replace('S{NNN}', 'S001').Replace('YYYY-MM-DD', $dateStr)
Write-FileUTF8 '7_Log/S001_log.md' $logContent

# --- Session_Registry.tsv ---
$tsvContent = "Session`tDate`tTitle`tStatus`tKey Finding`tArchive Path`r`nS001`t$dateStr`t[세션 제목]`t★ 활성`t-`t-`r`n"
Write-FileUTF8 '7_Log/2_Wiki/Session_Registry.tsv' $tsvContent

Write-Host '[4/6] Meta documents and config files created.'

# ================================================================
# 6. Git init & first commit (optional)
# ================================================================
Write-Host ''
if (Get-Command git -ErrorAction SilentlyContinue) {
    $gitChoice = Read-Host 'Initialize Git repository? [Y/n]'
    if ([string]::IsNullOrWhiteSpace($gitChoice) -or $gitChoice -match '^[Yy]') {
        try {
            git init
            git add .
            git commit -m 'chore: Initialize ELF v2.1 project structure'
            Write-Host '[5/6] Git initialized.'
        } catch {
            Write-Host "[5/6] Git init failed: $_"
        }
    } else {
        Write-Host '[5/6] Git initialization skipped.'
    }
} else {
    Write-Host '[5/6] Git not found — skipping Git initialization.'
}

Set-Location ..

Write-Host ''
Write-Host '============================================'
Write-Host "  [$projectName] ELF v2.1 project created!"
Write-Host "  Language: $projectLang"
Write-Host '============================================'
Write-Host '[6/6] Done!'

