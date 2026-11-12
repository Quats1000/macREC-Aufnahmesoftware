#!/bin/bash
# Schreibt dist/macREC-usb.img auf ein USB-Stick und passt die Groesse an.
#
#   ./tools/burn-mac.sh /dev/disk4
#
# Der Stick muss die Zielgroesse haben; das Skript passt die exFAT-Partition
# anschliessend auf die volle Kapazitaet an.
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
echo "==> Groesse wird angepasst"
# Das Skript aus dem Image ausfuehren: es partitioniert neu und formatiert exFAT.
HELPER="/Volumes/MACREC/GROESSE ANPASSEN.command"
if [ ! -f "$HELPER" ]; then
	echo "FEHLER: $HELPER nicht gefunden - ist der Stick gemountet?" >&2
	exit 1
fi
sudo bash "$HELPER"

echo
echo "Fertig."
