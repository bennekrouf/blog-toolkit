#Requires -Version 5.1
<#
.SYNOPSIS
    Blog Manager - Windows dependency installer.

.DESCRIPTION
    Installs the tools that blog-manager needs on Windows:
        Node.js  v20 LTS   (required by generate-blog-data.js when publishing posts)
        WebView2 runtime   (required by the Dioxus desktop shell — usually pre-installed on Windows 11)

    Already-installed tools at the correct version are skipped.
    Requires an internet connection. Run as Administrator for machine-wide installs,
    or without elevation for user-scope installs (WebView2 user bootstrap).
#>
param(
    [switch]$NoPrompt   # Set by the graphical installer — skips all Read-Host pauses
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$NODE_VERSION = '20.18.1'
$NODE_URL     = "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-x64.msi"
$TMP          = $env:TEMP

# ── Helpers ───────────────────────────────────────────────────────────────────
function Write-Step { param([string]$msg) Write-Host "`n>> $msg" -ForegroundColor Cyan }
function Write-Ok   { param([string]$msg) Write-Host "  OK  $msg" -ForegroundColor Green }
function Write-Skip { param([string]$msg) Write-Host "  --  $msg (already installed)" -ForegroundColor DarkGray }
function Write-Warn { param([string]$msg) Write-Host "  !!  $msg" -ForegroundColor Yellow }

function Refresh-Path {
    $machine = [System.Environment]::GetEnvironmentVariable('PATH', 'Machine')
    $user    = [System.Environment]::GetEnvironmentVariable('PATH', 'User')
    $env:PATH = "$machine;$user"
}

function Get-InstalledVersion {
    param([string]$Cmd, [string[]]$Arguments = @('--version'))
    try {
        $out = & $Cmd @Arguments 2>&1 | Out-String
        if ($out -match '(\d+\.\d+[\.\d]*)') { return $Matches[1] }
    } catch { }
    return $null
}

function Install-Msi {
    param([string]$Url, [string]$Label, [string]$OutFile)
    Write-Step "Installing $Label..."
    $msi = Join-Path $TMP $OutFile
    Write-Host "  Downloading $Url ..."
    (New-Object System.Net.WebClient).DownloadFile($Url, $msi)
    Write-Host "  Running installer (this may take a minute)..."
    $p = Start-Process msiexec.exe -ArgumentList "/i `"$msi`" /qn /norestart" -Wait -PassThru
    Remove-Item $msi -Force -ErrorAction SilentlyContinue
    if ($p.ExitCode -notin @(0, 3010)) {
        throw "$Label installer exited with code $($p.ExitCode)"
    }
    Refresh-Path
    Write-Ok "$Label installed"
}

Write-Host ""
Write-Host "  Blog Manager — Windows Setup" -ForegroundColor White
Write-Host "  ============================" -ForegroundColor White
Write-Host ""

# ── Node.js ───────────────────────────────────────────────────────────────────
Write-Step "Checking Node.js..."
$nodeVer = Get-InstalledVersion 'node'
if ($null -eq $nodeVer) {
    Install-Msi $NODE_URL "Node.js $NODE_VERSION" "node-setup.msi"
    Write-Ok "Node.js installed ($(Get-InstalledVersion 'node'))"
} else {
    $major = [int]($nodeVer -split '\.')[0]
    if ($major -lt 18) {
        Write-Warn "Node.js $nodeVer is too old — upgrading to $NODE_VERSION..."
        Install-Msi $NODE_URL "Node.js $NODE_VERSION" "node-setup.msi"
        Write-Ok "Node.js upgraded ($(Get-InstalledVersion 'node'))"
    } else {
        Write-Skip "Node.js $nodeVer"
    }
}

# ── WebView2 runtime ──────────────────────────────────────────────────────────
Write-Step "Checking WebView2 runtime..."

$wv2Key = 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
$wv2User = "HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

if ((Test-Path $wv2Key) -or (Test-Path $wv2User)) {
    Write-Skip "WebView2 runtime already installed"
} else {
    Write-Host "  Downloading WebView2 bootstrapper..."
    $wv2 = Join-Path $TMP "MicrosoftEdgeWebview2Setup.exe"
    (New-Object System.Net.WebClient).DownloadFile(
        "https://go.microsoft.com/fwlink/p/?LinkId=2124703", $wv2)
    Write-Host "  Installing WebView2 (user scope)..."
    Start-Process $wv2 -ArgumentList "/silent /install" -Wait
    Remove-Item $wv2 -Force -ErrorAction SilentlyContinue
    Write-Ok "WebView2 runtime installed"
}

# ── Done ──────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  Setup complete." -ForegroundColor Green
Write-Host "  Launch blog-manager.exe to start the app."
Write-Host ""

if (-not $NoPrompt) {
    Read-Host "Press Enter to close"
}
