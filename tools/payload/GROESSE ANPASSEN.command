#!/bin/bash
# GROESSE ANPASSEN.command - erweitert den USB-Stick auf die volle Kapazitaet.
#
# Wird nach dem Brennen des Images einmal ausgefuehrt (Doppelklick). Der Stick
# wird vollstaendig neu partitioniert und mit exFAT formatiert; die
# mitgelieferten Dateien werden danach wieder zurueckkopiert.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
set -euo pipefail

# Das Skript laeuft vom Stick aus - das haelt das Volume offen und verhindert
# das Aushangen. Deshalb wird es nach /tmp kopiert und die Kopie ausgefuehrt;
# die Originaldatei wird dadurch freigegeben.
if [ -z "${MACREC_RELOCATED:-}" ]; then
	MACREC_SOURCE="$(cd "$(dirname "$0")" && pwd -P)"
	RELOC_DIR="$(mktemp -d)"
	cp "$0" "$RELOC_DIR/groesse-anpassen.sh"
	chmod +x "$RELOC_DIR/groesse-anpassen.sh"
	export MACREC_RELOCATED=1 MACREC_SOURCE
	exec bash "$RELOC_DIR/groesse-anpassen.sh"
fi

# Pfad zum Volume: aus der Umgebung (vor der Verlagerung gemerkt) oder -
# falls das Skript direkt aus dem Volume gestartet wurde - aus dem eigenen Pfad.
MOUNT="${MACREC_SOURCE:-}"
if [ -z "$MOUNT" ] || [ ! -d "$MOUNT" ]; then
	MOUNT="$(cd "$(dirname "$0")" && pwd -P)"
fi
cd /tmp || exit 1

echo "macREC - Groesse anpassen"
echo "======================="
echo

# 1. Zielgeraet bestimmen: das physische Medium hinter dem aktuellen Volume.
#    diskutil info -plist liefert ParentWholeDisk (Whole-Disk) und TotalSize
#    zuverlaessig - die Textausgabe hat kein "Whole Disk"-Feld.
info_field() {
	diskutil info -plist "$1" | python3 -c '
import plistlib, sys
key = sys.argv[1]
print(plistlib.loads(sys.stdin.buffer.read().decode()).get(key, ""))
' "$2"
}

PARENT="$(info_field "$MOUNT" ParentWholeDisk)"
if [ -z "$PARENT" ]; then
	echo "FEHLER: Geraet zu $MOUNT nicht gefunden." >&2
	exit 1
fi
WHOLE="/dev/$PARENT"
SIZE_BYTES="$(info_field "$WHOLE" TotalSize)"
if [ -n "$SIZE_BYTES" ]; then
	SIZE="$(awk -v b="$SIZE_BYTES" 'BEGIN{printf "%.1f GB", b/1073741824}')"
else
	SIZE="unbekannt"
fi

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
#    Mit tar kopieren: tar schliesst alle Deskriptoren sofort, waehrend ditto
#    und cp das Volume offen halten und das Aushangen blockieren.
BACKUP="$(mktemp -d)"
echo "==> Dateien werden gesichert nach $BACKUP"
(cd "$MOUNT" && tar -cf - .) | (cd "$BACKUP" && tar -xf -) 2>/dev/null || true
rm -rf "$BACKUP"/._* "$BACKUP/.fseventsd" "$BACKUP/.Trashes" 2>/dev/null || true
if [ -z "$(ls -A "$BACKUP" 2>/dev/null)" ]; then
	echo "FEHLER: Die Dateien konnten nicht gesichert werden - abgebrochen." >&2
	exit 1
fi

# 3. Stick vollstaendig neu aufsetzen: eine Partition ueber die ganze Kapazitaet.
echo "==> Stick wird neu partitioniert und formatiert (exFAT)"
diskutil unmount "$MOUNT" >/dev/null 2>&1 || true
sleep 2
diskutil eraseDisk exFAT MACREC MBRFormat "$WHOLE" >/dev/null
sleep 3

# 4. Dateien zurueckkopieren. Das neue Volume haengt sich mit dem Label
#    MACREC unter /Volumes/MACREC ein. Der Whole-Disk selbst liefert keinen
#    Mount Point, deshalb wird das Partitions-Device ermittelt.
NEWMOUNT=""
for attempt in 1 2 3 4 5 6 7 8 9 10; do
	PARTDEV="$(diskutil list -plist | python3 -c '
import plistlib, sys
try:
    data = plistlib.loads(sys.stdin.buffer.read().decode())
except Exception:
    sys.exit(0)
whole = sys.argv[1].replace("/dev/", "")
for entry in data.get("AllDisksAndPartitions", []):
    for part in entry.get("Partitions", []):
        if part.get("VolumeName") == "MACREC" and part.get("MountPoint"):
            print(part["MountPoint"])
            sys.exit(0)
' "$WHOLE")"
	[ -n "$PARTDEV" ] && [ -d "$PARTDEV" ] && break
	sleep 1
done
if [ -z "$PARTDEV" ] || [ ! -d "$PARTDEV" ]; then
	echo "FEHLER: Das neu formatierte Volume wurde nicht gefunden." >&2
	exit 1
fi

echo "==> Dateien werden zurueckkopiert nach $PARTDEV"
(cd "$BACKUP" && tar -cf - .) | (cd "$PARTDEV" && tar -xf -) 2>/dev/null || true
# AppleDouble-Dateien (Metadaten von macOS) entfernen - sie sind auf FAT
# unnoetig und wuerden Windows stoeren. Auch im Backup-Ordner, damit sie
# nicht wieder zurueckkopiert werden.
find "$PARTDEV" -name '._*' -delete 2>/dev/null || true
find "$BACKUP" -name '._*' -delete 2>/dev/null || true
rm -rf "$PARTDEV/.fseventsd" "$PARTDEV/.Trashes" 2>/dev/null || true
chmod +x "$PARTDEV/GROESSE ANPASSEN.command" 2>/dev/null || true
rm -rf "$BACKUP"

echo
echo "Fertig. Der Stick nutzt jetzt die volle Kapazitaet."
echo "macREC starten:"
echo "  macOS:    \"$PARTDEV/macREC - Aufnahmesoftware.app\""
echo "  Windows:  macREC.exe"
