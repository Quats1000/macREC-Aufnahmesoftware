#!/bin/bash
# Schreibt dist/macREC-usb.img auf ein USB-Stick und passt die Groesse an.
#
#   sudo ./tools/burn-mac.sh /dev/disk4
#
# Der Stick nutzt danach die volle Kapazitaet; die Programme liegen als
# Dateien darauf und sind unter macOS wie Windows startbar.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMG="$ROOT/dist/macREC-usb.img"
DEVICE="${1:-}"

if [ -z "$DEVICE" ]; then
	echo "Aufruf: $0 /dev/diskN" >&2
	exit 2
fi
if [ ! -f "$IMG" ]; then
	echo "FEHLER: $IMG fehlt - erst './tools/build-img.sh' ausfuehren" >&2
	exit 1
fi

RDEVICE="/dev/rdisk$(basename "$DEVICE" | sed 's/disk//')"
echo "==> Ziel: $DEVICE"
diskutil list "$DEVICE" 2>/dev/null | head -6 || true
echo

echo "==> Image wird geschrieben (das dauert je nach Stick ein paar Minuten)"
sudo dd if="$IMG" of="$RDEVICE" bs=4m status=progress

echo
sudo "$ROOT/tools/groesse-anpassen.sh" "$DEVICE"

echo
echo "Fertig."
echo "macREC starten:"
echo "  macOS:    /Volumes/MACREC/macREC - Aufnahmesoftware.app"
echo "  Windows:  macREC.exe"
