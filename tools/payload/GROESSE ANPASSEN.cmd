@echo off
rem GROESSE ANPASSEN.cmd - erweitert den USB-Stick auf die volle Kapazitaet.
rem
rem Wird nach dem Brennen des Images einmal ausgefuehrt (Doppelklick). Der Stick
rem wird vollstaendig neu partitioniert und mit exFAT formatiert; die
rem mitgelieferten Dateien werden danach wieder zurueckkopiert.
rem
rem Achtung: alle Daten auf dem Stick werden geloescht.
setlocal
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0groesse-anpassen.ps1"
pause
