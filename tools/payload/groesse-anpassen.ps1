# groesse-anpassen.ps1 - erweitert den USB-Stick auf die volle Kapazitaet.
#
# Wird ueber GROESSE ANPASSEN.cmd aufgerufen. Der Stick wird vollstaendig neu
# partitioniert und mit exFAT formatiert; die mitgelieferten Dateien werden
# gesichert und danach wieder zurueckkopiert.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Das Skript laeuft vom Stick aus und wuerde das Volume blockieren - dann
# scheitert Clear-Disk mit "the disk is in use". Deshalb wird es nach %TEMP%
# kopiert und die Kopie ausgefuehrt; die Originaldatei wird dadurch frei.
# Erkannt wird die Kopie an der Umgebungsvariable und am Pfad selbst.
$isRelocated = ($env:MACREC_RELOCATED -eq '1') -or
	($MyInvocation.MyCommand.Path -like (Join-Path $env:TEMP '*'))

if (-not $isRelocated) {
	$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
	$principal = New-Object Security.Principal.WindowsPrincipal($identity)
	if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
		Write-Host 'Administratorrechte werden angefordert ...' -ForegroundColor Yellow
		Start-Process -FilePath 'powershell' -Verb RunAs -ArgumentList @(
			'-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$($MyInvocation.MyCommand.Path)`""
		)
		exit 0
	}

	# Nach %TEMP% kopieren und dort die eigentliche Arbeit erledigen. Der
	# Ordner bekommt einen eindeutigen Namen und wird danach wieder entfernt,
	# damit ein spaeterer Start nicht an einem alten Marker scheitert.
	$tempDir = Join-Path $env:TEMP ('macrec-relocated-' + [guid]::NewGuid().ToString('N'))
	New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
	Copy-Item -Path (Join-Path $scriptDir 'groesse-anpassen.ps1') -Destination $tempDir -Force
	Copy-Item -Path (Join-Path $scriptDir 'GROESSE ANPASSEN.cmd') -Destination $tempDir -Force
	$env:MACREC_RELOCATED = '1'
	& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $tempDir 'groesse-anpassen.ps1')
	$code = $LASTEXITCODE
	Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
	exit $code
}

Write-Host 'macREC - Groesse anpassen' -ForegroundColor Cyan
Write-Host '======================='
Write-Host ''

# 1. Volume mit dem Label MACREC finden (das ist der eben gebannte Stick).
$volume = Get-Volume | Where-Object { $_.FileSystemLabel -eq 'MACREC' -and $_.DriveLetter } |
	Select-Object -First 1
if (-not $volume) {
	$volume = Get-Volume | Where-Object { $_.DriveLetter -and (Test-Path "$($_.DriveLetter)\macREC.exe") } |
		Select-Object -First 1
}
if (-not $volume) {
	Write-Host 'FEHLER: Der macREC-Stick wurde nicht gefunden.' -ForegroundColor Red
	exit 1
}

$partition = Get-Partition -DriveLetter $volume.DriveLetter
$disk = Get-Disk -Number $partition.DiskNumber
Write-Host ("Stick:  Datentraeger {0} - {1:N1} GB" -f $disk.Number, ($disk.Size / 1GB))
Write-Host 'Der Stick wird vollstaendig neu eingerichtet.'
$answer = Read-Host 'Fortfahren? [j/N]'
if ($answer -notmatch '^[jJ]') {
	Write-Host 'Abgebrochen.'
	exit 0
}
Write-Host ''

# 2. Dateien, die erhalten bleiben muessen, in den Temp-Bereich sichern.
$backup = Join-Path $env:TEMP ('macrec-sicherung-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $backup | Out-Null
Write-Host "==> Dateien werden gesichert nach $backup"
Copy-Item -Path "$($volume.DriveLetter):\*" -Destination $backup -Recurse -Force
Get-ChildItem -Path $backup -Filter '._*' -Recurse -Force | Remove-Item -Force -ErrorAction SilentlyContinue
Remove-Item -Path (Join-Path $backup '.fseventsd') -Recurse -Force -ErrorAction SilentlyContinue

# 3. Stick vollstaendig neu aufsetzen: eine Partition ueber die ganze
#    Kapazitaet, formatiert als exFAT.
Write-Host '==> Stick wird neu partitioniert und formatiert (exFAT)'

# Das Volume muss ausgehängt sein, sonst blockiert es die Partitionierung.
Dismount-Volume -DriveLetter $volume.DriveLetter -Force -ErrorAction SilentlyContinue

# Frisch angeschlossene USB-Sticks stehen oft auf offline oder read-only -
# beides verhindert Clear-Disk.
Set-Disk -Number $disk.Number -IsOffline $false -ErrorAction SilentlyContinue
Set-Disk -Number $disk.Number -IsReadOnly $false -ErrorAction SilentlyContinue

# Clear-Disk entfernt saemtliche vorhandene Partitionen des Datentraegers.
Clear-Disk -Number $disk.Number -RemoveData -RemoveOEM -Confirm:$false
$newPartition = New-Partition -DiskNumber $disk.Number -UseMaximumSize -AssignDriveLetter
Format-Volume -Partition $newPartition -FileSystem exFAT -NewFileSystemLabel 'MACREC' -Force -Confirm:$false | Out-Null

$newLetter = (Get-Partition -DiskNumber $disk.Number | Select-Object -First 1).DriveLetter
if (-not $newLetter) {
	Write-Host 'FEHLER: Die neue Partition hat keinen Laufwerksbuchstaben.' -ForegroundColor Red
	exit 1
}

# 4. Dateien zurueckkopieren.
Write-Host "==> Dateien werden zurueckkopiert nach ${newLetter}:\"
Copy-Item -Path (Join-Path $backup '*') -Destination "${newLetter}:\" -Recurse -Force
Get-ChildItem -Path "${newLetter}:\" -Filter '._*' -Recurse -Force | Remove-Item -Force -ErrorAction SilentlyContinue
Remove-Item -Path "${newLetter}:\.fseventsd" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path $backup -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ''
Write-Host 'Fertig. Der Stick nutzt jetzt die volle Kapazitaet.' -ForegroundColor Green
Write-Host 'macREC starten:  macREC.exe'
