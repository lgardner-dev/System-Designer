$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $root
try {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'Rust/Cargo is required on the build machine. Install Rust stable and the Visual C++ build tools, then rerun.'
    }
    cargo fmt --all --check
    if ($LASTEXITCODE -ne 0) { throw 'Source is not formatted. Run cargo fmt --all.' }
    cargo test --locked --all-targets
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed. No installer was produced.' }
    cargo build --locked --release --bin system-designer
    if ($LASTEXITCODE -ne 0) { throw 'Native build failed. No installer was produced.' }

    $binary = (Resolve-Path 'target/release/system-designer.exe').Path

    # .cargo/config.toml links the CRT statically so the shipped binary does not
    # need the Visual C++ Redistributable. Verify it rather than assume it: a
    # VCRUNTIME140 import here means the app dies on a clean machine.
    $dumpbin = Get-ChildItem 'C:\Program Files*\Microsoft Visual Studio\*\*\VC\Tools\MSVC\*\bin\Hostx64\x64\dumpbin.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($dumpbin) {
        $deps = & $dumpbin.FullName /dependents $binary 2>$null
        if ($deps -match 'VCRUNTIME|MSVCP') {
            throw 'The executable imports the Visual C++ runtime DLLs. Confirm .cargo/config.toml is present and rebuild.'
        }
        Write-Host 'Verified: no Visual C++ Redistributable dependency.'
    } else {
        Write-Warning 'dumpbin not found; skipped the runtime-dependency check.'
    }

    # NSIS_HOME wins, then PATH, then the default install location.
    $makensis = $null
    foreach ($candidate in @(
        $(if ($env:NSIS_HOME) { Join-Path $env:NSIS_HOME 'makensis.exe' }),
        $((Get-Command makensis -ErrorAction SilentlyContinue).Source),
        "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
        "${env:ProgramFiles}\NSIS\makensis.exe"
    )) {
        if ($candidate -and (Test-Path $candidate)) { $makensis = $candidate; break }
    }
    if (-not $makensis) {
        throw 'The native executable was built, but NSIS is required to package the installer. Install it (choco install nsis) or set NSIS_HOME.'
    }

    $version = ([regex]::Match((Get-Content 'Cargo.toml' -Raw), '(?m)^version\s*=\s*"([^"]+)"')).Groups[1].Value
    # VIProductVersion accepts only four numeric fields, so drop any suffix such
    # as -beta and pad the result out to four parts.
    $numeric = ($version -split '[-+]')[0]
    $parts = @($numeric -split '\.') + @('0','0','0','0')
    $version4 = ($parts[0..3]) -join '.'

    New-Item -ItemType Directory -Force 'dist' | Out-Null
    $output = Join-Path $root 'dist/system-designer-setup.exe'
    & $makensis "/DAPP_VERSION=$version" "/DAPP_VERSION_4=$version4" "/DAPP_BINARY=$binary" "/DOUT_FILE=$output" 'packaging/windows/setup.nsi'
    if ($LASTEXITCODE -ne 0) { throw 'Installer packaging failed.' }
    Copy-Item 'Cargo.lock' 'dist/Cargo.lock'
    Get-FileHash $output -Algorithm SHA256 | Format-List
    Write-Host "Built unsigned installer: $output"
} finally { Pop-Location }
