#!/bin/bash
# GROESSE ANPASSEN.command - erweitert den USB-Stick auf die volle Kapazitaet.
#
# Wird nach dem Brennen des Images einmal ausgefuehrt. Der Stick wird dabei
# vollstaendig neu partitioniert und mit exFAT formatiert; die mitgelieferten
# Dateien werden danach wieder zurueckkopiert.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
set -euo pipefail

cd "$(dirname "$0")" || exit 1
MOUNT="$(pwd)"

echo "macREC - Groesse anpassen"
echo "======================="
echo

# 1. Zielgeraet bestimmen: das physische Medium hinter dem aktuellen Volume.
DEVICE="$(diskutil info "$MOUNT" | awk -F': *' '/Device Node/{print $2}' | head -1)"
WHOLE="$(diskutil info "$DEVICE" | awk -F': *' '/Whole Disk/{print $2}' | head -1)"
if [ -z "$WHOLE" ]; then
	echo "FEHLER: Geraet zu $MOUNT nicht gefunden." >&2
	exit 1
fi
SIZE="$(diskutil info "$WHOLE" | awk -F': *' '/Disk Size/{print $2}')"

echo "Stick:  $WHOLE ($SIZE)"
echo "Der Stick wird vollstaendig neu eingerichtet."
printf "Fortfahren? [j/N] "
read -r ANSWER
case "$ANSWER" in
	[jJ] | [yY]) ;;
	*) echo "Abgebrochen."; exit 0 ;;
esac
echo

# 2. Dateien, die erhalten bleiben muessen, in den Temp-Bereich sichern.
BACKUP="$(mktemp -d)"
echo "==> Dateien werden gesichert nach $BACKUP"
ditto "$MOUNT" "$BACKUP/inhalt" 2>/dev/null || cp -R "$MOUNT" "$BACKUP/inhalt"
rm -rf "$BACKUP/inhalt/._*" "$BACKUP/inhalt/.fseventsd" 2>/dev/null || true

# 3. Stick vollstaendig neu aufsetzen: eine Partition ueber die ganze Kapazitaet.
echo "==> Stick wird neu partitioniert und formatiert (exFAT)"
diskutil unmount "$MOUNT" >/dev/null 2>&1 || true
sleep 1
diskutil eraseDisk exFAT MACREC MBRFormat "$WHOLE" >/dev/null
sleep 2

# 4. Dateien zurueckkopieren.
NEWMOUNT="$(diskutil info "$WHOLE" | awk -F': *' '/Mount Point/{print $2}' | head -1)"
if [ -z "$NEWMOUNT" ] || [ ! -d "$NEWMOUNT" ]; then
	NEWMOUNT="/Volumes/MACREC"
	mkdir -p "$NEWMOUNT" 2>/dev/null || true
fi
echo "==> Dateien werden zurueckkopiert nach $NEWMOUNT"
ditto "$BACKUP/inhalt" "$NEWMOUNT" 2>/dev/null || cp -R "$BACKUP/inhalt/." "$NEWMOUNT/"
rm -rf "$NEWMOUNT/._*" "$NEWMOUNT/.fseventsd" 2>/dev/null || true
chmod +x "$NEWMOUNT/GROESSE ANPASSEN.command" 2>/dev/null || true
rm -rf "$BACKUP"

echo
echo "Fertig. Der Stick nutzt jetzt die volle Kapazitaet."
echo "macREC starten:"
echo "  macOS:    \"$NEWMOUNT/macREC - Aufnahmesoftware.app\""
echo "  Windows:  macREC.exe"
