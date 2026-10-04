# Install Rust
Write-Host "Downloading rustup-init..."
Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile "rustup-init.exe"
Write-Host "Installing Rust..."
.\rustup-init.exe -y --default-host x86_64-pc-windows-msvc
# Add cargo to PATH for this session
$env:Path += ";$HOME\.cargo\bin"
Write-Host "Rust installed."

# Install Visual Studio Build Tools
Write-Host "Downloading VS Build Tools..."
Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile "vs_buildtools.exe"
Write-Host "Installing VS Build Tools (this will pop up a UAC prompt and may take a while)..."
Start-Process -FilePath ".\vs_buildtools.exe" -ArgumentList "--quiet --wait --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" -Wait -Verb RunAs
Write-Host "VS Build Tools installed."
