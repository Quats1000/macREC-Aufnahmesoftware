#!/usr/bin/env python3
"""Schreibt einen MBR mit einer Primaerpartition (Typ 0x07) ab LBA 2048.

Aufruf: mbr.py <ziel-datei> <groesse-mb> [start-lba]
"""
import struct
import sys

SECTOR = 512
MBR_TYPE_EXFAT = 0x07


def main() -> int:
    if len(sys.argv) < 3:
        print("Aufruf: mbr.py <ziel> <groesse-mb> [start-lba]", file=sys.stderr)
        return 2

    path = sys.argv[1]
    size_mb = int(sys.argv[2])
    start = int(sys.argv[3]) if len(sys.argv) > 3 else 2048

    total = size_mb * 1024 * 1024 // SECTOR
    if start >= total:
        print("Partition passt nicht in das Medium", file=sys.stderr)
        return 1

    mbr = bytearray(SECTOR)
    entry = 0x1BE
    mbr[entry + 0] = 0x00                              # Boot-Flag
    mbr[entry + 1:entry + 4] = bytes([0xFE, 0xFF, 0xFF])  # Start-CHS (LBA-Modus)
    mbr[entry + 4] = MBR_TYPE_EXFAT                    # Typ: Windows_NTFS / exFAT
    mbr[entry + 5:entry + 8] = bytes([0xFE, 0xFF, 0xFF])  # End-CHS
    mbr[entry + 8:entry + 12] = struct.pack("<I", start)
    mbr[entry + 12:entry + 16] = struct.pack("<I", total - start)
    mbr[0x1FE:0x200] = b"\x55\xAA"

    with open(path, "r+b") as handle:
        handle.seek(0)
        handle.write(mbr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
