# Groesse anpassen - erweitert den USB-Stick auf die volle Kapazitaet.
#
# Wird nach dem Brennen des Images einmal ausgefuehrt (Doppelklick auf
# GROESSE ANPASSEN.cmd). Der Stick wird vollstaendig neu partitioniert und
# mit exFAT formatiert; die mitgelieferten Dateien werden danach wieder
# zurueckkopiert.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

Write-Host 'macREC - Groesse anpassen' -ForegroundColor Cyan
Write-Host '======================='
Write-Host ''

# 1. Administratorrechte pruefen und bei Bedarf mit Hoeherfuehrung neu starten.
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
	Write-Host 'Administratorrechte werden angefordert ...' -ForegroundColor Yellow
	Start-Process -FilePath 'powershell' -Verb RunAs -ArgumentList @(
		'-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$($MyInvocation.MyCommand.Path)`""
	)
	exit 0
}

# 2. Physisches Geraet hinter dem aktuellen Volume ermitteln.
$volume = Get-Volume | Where-Object { $_.FileSystemLabel -eq 'MACREC' -and $_.DriveLetter } |
	Select-Object -First 1
if (-not $volume) {
	$volume = Get-Volume | Where-Object { $_.DriveLetter -and (Test-Path "$($_.DriveLetter)\macREC.exe") } |
		Select-Object -First 1
}
if (-not $volume) {
	Write-Host 'FEHLER: Der macREC-Stick wurde nicht gefunden.' -ForegroundColor Red
	Read-Host 'Zum Beenden Eingabetaste druecken'
	exit 1
}

$partition = Get-Partition -DriveLetter $volume.DriveLetter
$disk = Get-Disk -Number $partition.DiskNumber

Write-Host "Stick:  Disk $($disk.Number) - $($disk.BusType) - $([math]::Round($disk.Size / 1GB, 1)) GB"
Write-Host 'Der Stick wird vollstaendig neu eingerichtet.'
$answer = Read-Host 'Fortfahren? [j/N]'
if ($answer -notmatch '^[jJ]') {
	Write-Host 'Abgebrochen.'
	Read-Host 'Zum Beenden Eingabetaste druecken'
	exit 0
}
Write-Host ''

# 3. Dateien, die erhalten bleiben muessen, in den Temp-Bereich sichern.
$backup = Join-Path $env:TEMP ('macrec-sicherung-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $backup | Out-Null
Write-Host "==> Dateien werden gesichert nach $backup"
Copy-Item -Path (Join-Path $root '*') -Destination $backup -Recurse -Force
Get-ChildItem -Path $backup -Filter '._*' -Recurse -Force | Remove-Item -Force -ErrorAction SilentlyContinue
Remove-Item -Path (Join-Path $backup '.fseventsd') -Recurse -Force -ErrorAction SilentlyContinue

# 4. Stick vollstaendig neu aufsetzen: eine Partition ueber die ganze
#    Kapazitaet, formatiert als exFAT.
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

# 5. Dateien zurueckkopieren.
Write-Host "==> Dateien werden zurueckkopiert nach ${newLetter}:\"
Copy-Item -Path (Join-Path $backup '*') -Destination "${newLetter}:\" -Recurse -Force
Get-ChildItem -Path "${newLetter}:\" -Filter '._*' -Recurse -Force | Remove-Item -Force -ErrorAction SilentlyContinue
Remove-Item -Path "${newLetter}:\.fseventsd" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path $backup -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ''
Write-Host 'Fertig. Der Stick nutzt jetzt die volle Kapazitaet.' -ForegroundColor Green
Write-Host 'macREC starten:  macREC.exe'
