# groesse-anpassen.ps1 - erweitert den USB-Stick auf die volle Kapazitaet.
#
# Wird ueber GROESSE ANPASSEN.cmd aufgerufen. Der Stick wird vollstaendig neu
# partitioniert und mit exFAT formatiert; die mitgelieferten Dateien werden
# gesichert und danach wieder zurueckkopiert.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
$ErrorActionPreference = 'Stop'

# Administratorrechte pruefen und bei Bedarf mit Hoeherfuehrung neu starten.
# Wichtig: das Skript laeuft vom Stick aus und wuerde das Volume blockieren.
# Deshalb wird es vor der Formatierung nach %TEMP% kopiert und die Kopie
# ausgefuehrt - dann ist der Stick frei.
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$isRelocated = Test-Path -LiteralPath (Join-Path $env:TEMP 'macrec-relocated')

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

	# Nach %TEMP% kopieren und dort die eigentliche Arbeit erledigen.
	$tempDir = Join-Path $env:TEMP 'macrec-relocated'
	New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
	Copy-Item -Path (Join-Path $scriptDir 'groesse-anpassen.ps1') -Destination $tempDir -Force
	Copy-Item -Path (Join-Path $scriptDir 'GROESSE ANPASSEN.cmd') -Destination $tempDir -Force
	& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $tempDir 'groesse-anpassen.ps1')
	exit $LASTEXITCODE
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
