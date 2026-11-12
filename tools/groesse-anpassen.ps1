# groesse-anpassen.ps1 - erweitert ein bereits geschriebenes macREC-Abbild
# auf die volle Kapazitaet des USB-Sticks.
#
#   powershell -ExecutionPolicy Bypass -File tools\groesse-anpassen.ps1 -Device 2
#
# Wird von tools/burn-windows.ps1 nach dem Schreiben des Images aufgerufen.
# Der Stick wird vollstaendig neu partitioniert und mit exFAT formatiert; die
# vorhandenen Dateien werden gesichert und danach wieder zurueckgelegt.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
param(
	[int]$Device = 0
)

$ErrorActionPreference = 'Stop'

if ($Device -eq 0) {
	Write-Host 'FEHLER: -Device (Datentraegernummer) fehlt.' -ForegroundColor Red
	exit 2
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

Write-Host "==> Groesse wird angepasst (Datentraeger $Device)"

$disk = Get-Disk -Number $Device

# 1. Volume mit dem Label MACREC finden und dessen Dateien sichern.
$volume = Get-Volume | Where-Object { $_.FileSystemLabel -eq 'MACREC' -and $_.DriveLetter } |
	Select-Object -First 1
if (-not $volume) {
	$volume = Get-Volume | Where-Object { $_.DriveLetter -and (Test-Path "$($_.DriveLetter)\macREC.exe") } |
		Select-Object -First 1
}

$backup = Join-Path $env:TEMP ('macrec-sicherung-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $backup | Out-Null
if ($volume) {
	Write-Host "==> Dateien werden gesichert nach $backup"
	Copy-Item -Path "$($volume.DriveLetter):\*" -Destination $backup -Recurse -Force
	Get-ChildItem -Path $backup -Filter '._*' -Recurse -Force | Remove-Item -Force -ErrorAction SilentlyContinue
	Remove-Item -Path (Join-Path $backup '.fseventsd') -Recurse -Force -ErrorAction SilentlyContinue
} else {
	Write-Host '    (kein Volume gefunden - es wird nur formatiert)'
}

# 2. Stick neu partitionieren und formatieren.
Write-Host '==> Stick wird neu partitioniert und formatiert (exFAT)'
# Clear-Disk entfernt saemtliche vorhandene Partitionen des Datentraegers.
Clear-Disk -Number $disk.Number -RemoveData -RemoveOEM -Confirm:$false
$newPartition = New-Partition -DiskNumber $disk.Number -UseMaximumSize -AssignDriveLetter
Format-Volume -Partition $newPartition -FileSystem exFAT -NewFileSystemLabel 'MACREC' -Force -Confirm:$false | Out-Null

$newLetter = (Get-Partition -DiskNumber $disk.Number | Select-Object -First 1).DriveLetter
if (-not $newLetter) {
	Write-Host 'FEHLER: Die neue Partition hat keinen Laufwerksbuchstaben.' -ForegroundColor Red
	exit 1
}

# 3. Dateien zurueckkopieren.
if (Get-ChildItem -Path $backup -Force -ErrorAction SilentlyContinue) {
	Write-Host "==> Dateien werden zurueckkopiert nach ${newLetter}:\"
	Copy-Item -Path (Join-Path $backup '*') -Destination "${newLetter}:\" -Recurse -Force
	Get-ChildItem -Path "${newLetter}:\" -Filter '._*' -Recurse -Force | Remove-Item -Force -ErrorAction SilentlyContinue
	Remove-Item -Path "${newLetter}:\.fseventsd" -Recurse -Force -ErrorAction SilentlyContinue
}
Remove-Item -Path $backup -Recurse -Force -ErrorAction SilentlyContinue

Write-Host '==> Fertig. Der Stick nutzt jetzt die volle Kapazitaet.' -ForegroundColor Green
