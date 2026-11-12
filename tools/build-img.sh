#!/bin/bash
# Erzeugt dist/macREC-usb.img: ein Roh-Abbild (MBR + exFAT) mit dem macREC-Stick.
#
#   ./tools/build-img.sh [Groesse-in-MB]
#
# Das Abbild kann mit jedem Brennwerkzeug auf einen USB-Stick geschrieben werden
# (dd, balenaEtcher, Rufus im DD-Modus, Win32DiskImager ...).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SIZE_MB="${1:-512}"
STAGE="$(mktemp -d)"
WORK="$(mktemp -d)"
trap 'rm -rf "$STAGE" "$WORK"' EXIT

echo "==> Universelles macOS-Binary (arm64 + x86_64)"
ARM="target/release/screenrecv"
X86="target/x86_64-apple-darwin/release/screenrecv"
if [ ! -x "$ARM" ]; then
	echo "    arm64-Binary fehlt - bitte 'cargo build --release' ausfuehren" >&2
	exit 1
fi
if [ -x "$X86" ]; then
	lipo -create -output "$WORK/screenrecv" "$ARM" "$X86"
else
	echo "    x86_64-Binary fehlt - nur arm64 (Intel-Macs koennen es nicht starten)"
	cp "$ARM" "$WORK/screenrecv"
fi
lipo -info "$WORK/screenrecv"

echo "==> App-Bundle bauen"
(cd crates/packager && go run . app \
	--bin "$WORK/screenrecv" \
	--icon "$ROOT/crates/screenrecv/resources/app-icon.png" \
	--out "$STAGE/macREC - Aufnahmesoftware.app")

echo "==> Windows-Programm und README kopieren"
cp dist/macREC.exe "$STAGE/"
cp dist/README.txt "$STAGE/"

echo "==> Hilfsskripte fuer die Groessenanpassung beilegen"
cp tools/payload/"GROESSE ANPASSEN.command" "$STAGE/"
cp tools/payload/"GROESSE ANPASSEN.cmd" "$STAGE/"
cp tools/payload/groesse-anpassen.ps1 "$STAGE/"

# Legt ein Roh-Abbild mit MBR und einer exFAT- bzw. FAT32-Partition an.
build_image() {
	local img="$1" size_mb="$2" fstype="$3"

	rm -f "$img"
	truncate -s "${size_mb}M" "$img"
	python3 tools/mbr.py "$img" "$size_mb"

	# Das Abbild als Geraet einbinden, das Dateisystem direkt auf die Partition legen.
	local dev part
	dev="$(hdiutil attach -nomount "$img" | awk 'NR==1{print $1}')"
	# Partitions-Device: aus dem Namen des Whole-Disk ableiten (disk9 -> disk9s1).
	part="/dev/$(basename "$dev")s1"
	if [ ! -b "$part" ]; then
		echo "    FEHLER: Partition $part nicht gefunden" >&2
		exit 1
	fi
	echo "    Geraet=$dev Partition=$part Dateisystem=$fstype"
	sleep 1
	if [ "$fstype" = "fat32" ]; then
		# newfs_msdos erzeugt kein korrektes FAT32 (FS-Typ-Feld fehlt).
		# hdiutil legt dagegen ein vollstaendiges FAT32 mit Label an.
		hdiutil detach "$dev" >/dev/null 2>&1
		sleep 1
		rm -f "$img.dmg"
		hdiutil create -size "${size_mb}m" -fs "MS-DOS FAT32" -volname MACREC -ov "$img.dmg" >/dev/null
		mv "$img.dmg" "$img"
		sleep 1
		dev="$(hdiutil attach -nomount "$img" | awk 'NR==1{print $1}')"
		part="/dev/$(basename "$dev")s1"
		sleep 1
	else
		newfs_exfat -v MACREC "$part" >/dev/null
		sleep 1
	fi

	# Volume einhaengen und den Mount-Punkt ermitteln.
	diskutil mount "$part" >/dev/null 2>&1 || true
	sleep 1
	local mnt
	mnt="$(diskutil info "$part" | awk -F': *' '/Mount Point/{print $2}' | head -1)"
	if [ -z "$mnt" ] || [ ! -d "$mnt" ]; then
		echo "    FEHLER: Partition $part liess sich nicht einhaengen" >&2
		exit 1
	fi
	rsync -a --exclude '._*' --exclude '.fseventsd' "$STAGE/" "$mnt/"
	# AppleDouble-Dateien (Metadaten von macOS) entfernen - sie sind auf FAT
	# unnoetig und wuerden Windows/Mac staeren.
	find "$mnt" -name '._*' -delete 2>/dev/null || true
	rm -rf "$mnt/.fseventsd" 2>/dev/null || true
	hdiutil detach "$dev" >/dev/null
	sleep 1
}

echo "==> Abbild anlegen (${SIZE_MB} MB, MBR + exFAT)"
IMG="$ROOT/dist/macREC-usb.img"
build_image "$IMG" "$SIZE_MB" exfat
echo "==> Fertig: dist/macREC-usb.img ($(du -h "$IMG" | cut -f1))"

echo "==> Zweites Abbild (FAT32, fuer aeltere Systeme)"
IMG32="$ROOT/dist/macREC-usb-fat32.img"
build_image "$IMG32" "$SIZE_MB" fat32
echo "==> Fertig: dist/macREC-usb-fat32.img ($(du -h "$IMG32" | cut -f1))"

echo
echo "Schreiben mit:  sudo dd if=dist/macREC-usb.img of=/dev/rdiskN bs=4m"
