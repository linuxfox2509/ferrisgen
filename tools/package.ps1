# FerrisGen packaging script (Windows)
# Builds release binaries and packages them into a ZIP under dist\

param(
    [switch] $IncludeReadme = $true,
    [string] $OutName = $("FerrisGen-Windows-" + (Get-Date -Format yyyyMMddHHmmss) + ".zip")
)

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
# Assume script in tools\, workspace root is parent
$workspace = Resolve-Path "$root\.."
Set-Location $workspace

Write-Host "Workspace: $workspace"

# Ensure release builds
Write-Host "Building release binaries..."
cargo build --release -p ferrisgen-cli -p ferrisgen-gui -p ferrisgen-installer
if ($LASTEXITCODE -ne 0) { Write-Error "Build failed"; exit $LASTEXITCODE }

$target = Join-Path $workspace 'target\release'
$distTmp = Join-Path $workspace 'dist\FerrisGenBundle'
if (Test-Path $distTmp) { Remove-Item -Recurse -Force $distTmp }
New-Item -ItemType Directory -Path $distTmp | Out-Null

# List of binaries to include
$binaries = @('ferrisgen-cli.exe','ferrisgen-gui.exe','ferrisgen-installer.exe')

foreach ($b in $binaries) {
    $src = Join-Path $target $b
    if (-Not (Test-Path $src)) {
        Write-Error "Missing binary: $src"; exit 1
    }
    Copy-Item $src -Destination $distTmp
}

if ($IncludeReadme) {
    if (Test-Path "README.md") { Copy-Item "README.md" -Destination $distTmp }
    if (Test-Path "LICENSE") { Copy-Item "LICENSE" -Destination $distTmp }
}

# Optionally include installer readme
if (Test-Path "tools\installer\README.md") { Copy-Item "tools\installer\README.md" -Destination $distTmp }

$dist = Join-Path $workspace 'dist'
if (-Not (Test-Path $dist)) { New-Item -ItemType Directory -Path $dist | Out-Null }

$out = Join-Path $dist $OutName
if (Test-Path $out) { Remove-Item $out -Force }

Write-Host "Generating file checksums..."
$files = Get-ChildItem -Path $distTmp -File
$checksums = foreach ($f in $files) { $h = (Get-FileHash -Algorithm SHA256 $f.FullName).Hash; "${h}  $($f.Name)" }
$checksums | Out-File -Encoding UTF8 -FilePath (Join-Path $distTmp "SHA256SUMS.txt")

Write-Host "Creating ZIP: $out"
Compress-Archive -Path (Join-Path $distTmp '*') -DestinationPath $out -Force

# Generate checksum for the ZIP
$zipHash = (Get-FileHash -Algorithm SHA256 $out).Hash
$zipBase = Split-Path $out -Leaf
$zipShaFile = $out + ".sha256"
"${zipHash}  ${zipBase}" | Out-File -Encoding UTF8 -FilePath $zipShaFile

# Clean up temp folder
Remove-Item -Recurse -Force $distTmp

Write-Host "Packaging complete. Output: $out"
Write-Host "ZIP checksum written to: $zipShaFile"