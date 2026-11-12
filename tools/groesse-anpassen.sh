#!/bin/bash
# groesse-anpassen.sh - erweitert ein bereits geschriebenes macREC-Abbild auf
# die volle Kapazitaet des USB-Sticks.
#
#   ./tools/groesse-anpassen.sh /dev/disk4
#
# Wird von tools/burn-mac.sh nach dem Schreiben des Images aufgerufen. Der
# Stick wird vollstaendig neu partitioniert und mit exFAT formatiert; die
# vorhandenen Dateien werden gesichert und danach wieder zurueckgelegt.
#
# Achtung: alle Daten auf dem Stick werden geloescht.
set -euo pipefail

DEVICE="${1:-}"
if [ -z "$DEVICE" ]; then
	echo "Aufruf: $0 /dev/diskN" >&2
	exit 2
fi
# rdisk-Schreibweise (z. B. /dev/rdisk4) auf das Block-Device normalisieren.
DEVICE="$(echo "$DEVICE" | sed 's|/dev/rdisk|/dev/disk|')"

echo "==> Groesse wird angepasst ($DEVICE)"

# 1. Aktuell gemountetes Volume des Sticks finden (Label MACREC). Gefiltert
#    auf den Ziel-Datentraeger - ein zweiter angeschlossener Stick mit dem
#    gleichen Label darf nicht versehentlich gesichert werden.
MOUNT=""
for attempt in 1 2 3 4 5; do
	MOUNT="$(diskutil list -plist | python3 -c '
import plistlib, sys
try:
    data = plistlib.loads(sys.stdin.buffer.read().decode())
except Exception:
    sys.exit(0)
whole = sys.argv[1].replace("/dev/", "")
for entry in data.get("AllDisksAndPartitions", []):
    if entry.get("DeviceIdentifier") != whole:
        continue
    for part in entry.get("Partitions", []):
        if part.get("VolumeName") == "MACREC" and part.get("MountPoint"):
            print(part["MountPoint"])
            sys.exit(0)
' "$DEVICE")"
	[ -n "$MOUNT" ] && [ -d "$MOUNT" ] && break
	sleep 1
done

# 2. Dateien sichern.
BACKUP="$(mktemp -d)"
if [ -n "$MOUNT" ] && [ -d "$MOUNT" ]; then
	echo "==> Dateien werden gesichert nach $BACKUP"
	(cd "$MOUNT" && tar -cf - .) | (cd "$BACKUP" && tar -xf -) 2>/dev/null || true
	rm -rf "$BACKUP"/._* "$BACKUP/.fseventsd" "$BACKUP/.Trashes" 2>/dev/null || true
else
	echo "    (kein Volume gemountet - es wird nur formatiert)"
fi

# 3. Stick neu partitionieren und formatieren.
if [ -n "$MOUNT" ] && [ -d "$MOUNT" ]; then
	diskutil unmount "$MOUNT" >/dev/null 2>&1 || true
	sleep 2
fi
diskutil eraseDisk exFAT MACREC MBRFormat "$DEVICE" >/dev/null
sleep 3

# 4. Dateien zurueckkopieren.
NEWMOUNT=""
for attempt in 1 2 3 4 5 6 7 8 9 10; do
	PARTDEV="$(diskutil list -plist | python3 -c '
import plistlib, sys
try:
    data = plistlib.loads(sys.stdin.buffer.read().decode())
except Exception:
    sys.exit(0)
whole = sys.argv[1].replace("/dev/", "")
for name in data.get("AllDisks", []):
    if name.startswith(whole + "s"):
        print("/dev/" + name)
        break
' "$DEVICE")"
	if [ -n "$PARTDEV" ]; then
		NEWMOUNT="$(diskutil info -plist "$PARTDEV" 2>/dev/null | python3 -c '
import plistlib, sys
try:
    data = plistlib.loads(sys.stdin.buffer.read().decode())
except Exception:
    sys.exit(0)
print(data.get("MountPoint", ""))
')"
	fi
	[ -n "$NEWMOUNT" ] && [ -d "$NEWMOUNT" ] && break
	sleep 1
done

if [ -z "$NEWMOUNT" ] || [ ! -d "$NEWMOUNT" ]; then
	echo "FEHLER: Das neu formatierte Volume wurde nicht gefunden." >&2
	exit 1
fi

if [ -n "$(ls -A "$BACKUP" 2>/dev/null)" ]; then
	echo "==> Dateien werden zurueckkopiert nach $NEWMOUNT"
	(cd "$BACKUP" && tar -cf - .) | (cd "$NEWMOUNT" && tar -xf -) 2>/dev/null || true
	find "$NEWMOUNT" -name '._*' -delete 2>/dev/null || true
	rm -rf "$NEWMOUNT/.fseventsd" "$NEWMOUNT/.Trashes" 2>/dev/null || true
fi
rm -rf "$BACKUP"

echo "==> Fertig. Der Stick nutzt jetzt die volle Kapazitaet."
