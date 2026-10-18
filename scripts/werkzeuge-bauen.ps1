# Holt die Werkzeuge, die die App mitliefert, nach build\werkzeuge\ (Windows x64).
# macOS und Linux: scripts/werkzeuge-bauen.sh. Versionen und Prüfsummen: werkzeuge.lock.
#
# Ergebnis (nicht im Repo):
#   build\werkzeuge\bin\{tinymist,vale,uv,typst2vast}.exe
#   build\werkzeuge\ltex-ls-plus\        LTeX+ mit eigener Java-Laufzeit
#   build\werkzeuge\vale\                Konfiguration und Stil FHDW
#   build\werkzeuge\typst\fhdw-arbeit\   Vorlage (wird beim Start installiert)
#   build\werkzeuge\licenses\            Lizenztexte
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$lock = Join-Path $root 'werkzeuge.lock'
$out = Join-Path $root 'build\werkzeuge'
$cache = Join-Path $root 'build\cache'
$target = 'windows-x64'

if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force "$out\bin", "$out\licenses", $cache | Out-Null

# Datei laden (falls nicht im Cache) und gegen die Prüfsumme aus werkzeuge.lock prüfen.
function Get-Verified($name, $url, $sha) {
  $file = Join-Path $cache "$name-$target-$([IO.Path]::GetFileName($url))"
  if (-not (Test-Path $file)) {
    Write-Host "Lade $name ..."
    Invoke-WebRequest -Uri $url -OutFile "$file.part"
    Move-Item "$file.part" $file
  }
  $actual = (Get-FileHash -Algorithm SHA256 $file).Hash.ToLower()
  if ($actual -ne $sha) {
    Remove-Item $file
    throw "Prüfsumme von $name stimmt nicht (erwartet $sha, erhalten $actual)."
  }
  return $file
}

foreach ($line in Get-Content $lock) {
  $f = $line -split '\s+'
  if ($f.Count -lt 5 -or $f[0] -ne 'download' -or $f[2] -ne $target) { continue }
  $name = $f[1]; $sha = $f[3]; $url = $f[4]
  $file = Get-Verified $name $url $sha
  $tmp = Join-Path $cache "entpackt-$name"
  if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
  New-Item -ItemType Directory $tmp | Out-Null
  switch ($name) {
    'tinymist' { Copy-Item $file "$out\bin\tinymist.exe" }
    'vale' { Expand-Archive $file $tmp; Copy-Item "$tmp\vale.exe" "$out\bin\vale.exe" }
    'uv' { Expand-Archive $file $tmp; Copy-Item (Get-ChildItem $tmp -Recurse -Filter uv.exe | Select-Object -First 1).FullName "$out\bin\uv.exe" }
    'ltex-ls-plus' {
      Expand-Archive $file $tmp
      New-Item -ItemType Directory "$out\ltex-ls-plus" | Out-Null
      $inner = Get-ChildItem $tmp -Directory | Select-Object -First 1
      Copy-Item "$($inner.FullName)\*" "$out\ltex-ls-plus" -Recurse
    }
  }
  # Lizenztexte aus dem Paket beilegen.
  New-Item -ItemType Directory -Force "$out\licenses\$name" | Out-Null
  Get-ChildItem $tmp -Recurse -File -Include 'LICENSE*', 'NOTICE*', 'COPYING*' -ErrorAction SilentlyContinue |
    ForEach-Object { Copy-Item $_.FullName "$out\licenses\$name\" }
  Remove-Item -Recurse -Force $tmp
}

# Typst-Pakete, die die Vorlage braucht (die App lädt selbst keine Pakete nach).
foreach ($line in Get-Content $lock) {
  $f = $line -split '\s+'
  if ($f.Count -lt 5 -or $f[0] -ne 'package') { continue }
  $spec = $f[1]; $version = $f[2]; $sha = $f[3]; $url = $f[4]
  $file = Join-Path $cache (($spec -replace '/', '-') + "-$version.tar.gz")
  if (-not (Test-Path $file)) { Invoke-WebRequest -Uri $url -OutFile $file }
  $actual = (Get-FileHash -Algorithm SHA256 $file).Hash.ToLower()
  if ($sha -eq '-') {
    Write-Warning "Für $spec $version ist keine Prüfsumme festgelegt. Berechnet: $actual (in werkzeuge.lock eintragen)."
  } elseif ($actual -ne $sha) {
    Remove-Item $file
    throw "Prüfsumme von $spec stimmt nicht (erwartet $sha, erhalten $actual)."
  }
  $dest = Join-Path $out "typst\packages\$spec\$version"
  New-Item -ItemType Directory -Force $dest | Out-Null
  tar -xzf $file -C $dest
  if ($LASTEXITCODE -ne 0) { throw "Entpacken von $spec fehlgeschlagen." }
}

# typst2vast (Typst → Text für Vale): aus dem Quelltext bauen.
$crate = (Get-Content $lock | Where-Object { $_ -match '^crate\s+typst2vast\s+' } | ForEach-Object { ($_ -split '\s+')[2] })
Write-Host "Baue typst2vast $crate ..."
cargo install typst2vast --version $crate --locked --root "$cache\typst2vast"
if ($LASTEXITCODE -ne 0) { throw 'cargo install typst2vast ist fehlgeschlagen.' }
Copy-Item "$cache\typst2vast\bin\typst2vast.exe" "$out\bin\typst2vast.exe"

# Konfiguration, Vorlage, Lizenzen, Versionen.
New-Item -ItemType Directory -Force "$out\vale", "$out\typst" | Out-Null
Copy-Item "$root\sidecars\vale\*" "$out\vale" -Recurse -Force
# Ohne Punkt am Namen, damit kein Paketierer die „versteckte“ Datei auslässt.
Move-Item "$out\vale\.vale.ini" "$out\vale\vale.ini" -Force
Copy-Item "$root\typst\fhdw-arbeit" "$out\typst\fhdw-arbeit" -Recurse
Copy-Item "$root\docs\werkzeuge-lizenzen.md" "$out\licenses\README.md"
Copy-Item $lock "$out\VERSIONEN.txt"

Write-Host "Fertig: $out"
