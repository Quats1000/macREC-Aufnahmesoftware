# Schreibt dist/macREC-usb.img auf einen USB-Stick und passt die Groesse an.
#
#   powershell -ExecutionPolicy Bypass -File tools\burn-windows.ps1 -Device 2
#
# -Device ist die Datentraegernummer aus der Datentraegerverwaltung (2, 3, ...).
param(
	[int]$Device = 0
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$img = Join-Path (Split-Path -Parent $root) 'dist\macREC-usb.img'

if (-not (Test-Path $img)) {
	Write-Host "FEHLER: $img fehlt - erst 'tools\build-img.sh' ausfuehren" -ForegroundColor Red
	exit 1
}

# Administratorrechte pruefen und bei Bedarf mit Hoeherfuehrung neu starten.
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
	Write-Host 'Administratorrechte werden angefordert ...' -ForegroundColor Yellow
	Start-Process -FilePath 'powershell' -Verb RunAs -ArgumentList @(
		'-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$($MyInvocation.MyCommand.Path)`"", '-Device', $Device
	)
	exit 0
}

if ($Device -eq 0) {
	Write-Host 'Verfuegbare Datentraeger:'
	Get-Disk | Select-Object Number, FriendlyName, @{n = 'GB'; e = { [math]::Round($_.Size / 1GB, 1) } } | Format-Table | Out-String | Write-Host
	Write-Host 'Datentraegernummer angeben:  -Device 2'
	Read-Host 'Zum Beenden Eingabetaste druecken'
	exit 1
}

$disk = Get-Disk -Number $Device
Write-Host "==> Ziel: Datentraeger $Device - $($disk.FriendlyName) - $([math]::Round($disk.Size / 1GB, 1)) GB"
$answer = Read-Host 'Fortfahren? [j/N]'
if ($answer -notmatch '^[jJ]') {
	Write-Host 'Abgebrochen.'
	Read-Host 'Zum Beenden Eingabetaste druecken'
	exit 0
}

Write-Host ''
Write-Host '==> Image wird geschrieben (das dauert je nach Stick ein paar Minuten)'
$source = [System.IO.File]::OpenRead($img)
$target = [System.IO.File]::Open("\\.\PhysicalDrive$Device", [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
try {
	$buffer = New-Object byte[] (4MB)
	while (($read = $source.Read($buffer, 0, $buffer.Length)) -gt 0) {
		$target.Write($buffer, 0, $read)
	}
	$target.Flush($true)
} finally {
	$source.Dispose()
	$target.Dispose()
}

Write-Host ''
Write-Host '==> Groesse wird angepasst'
$script = Join-Path $root 'tools\groesse-anpassen.ps1'
& powershell -NoProfile -ExecutionPolicy Bypass -File $script -Device $Device

Write-Host ''
Write-Host 'Fertig.'
Read-Host 'Zum Beenden Eingabetaste druecken'
