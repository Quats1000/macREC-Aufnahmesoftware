# GROESSE ANPASSEN.cmd - erweitert den USB-Stick auf die volle Kapazitaet.
#
# Wird nach dem Brennen des Images einmal ausgefuehrt (Doppelklick). Der Stick
# wird vollstaendig neu partitioniert und mit exFAT formatiert; die
# mitgelieferten Dateien werden danach wieder zurueckkopiert.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
@echo off
setlocal
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0groesse-anpassen.ps1"
pause
