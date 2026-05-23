#!/usr/bin/env python3
"""Genera el icono de marca para el instalador Windows (PNG + ICO)."""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "assets" / "branding"
PNG_PATH = OUT_DIR / "aurexalis-icon.png"
ICO_PATH = OUT_DIR / "aurexalis.ico"

BG = (8, 5, 15, 255)
PURPLE = (111, 56, 255, 255)
RED = (255, 31, 85, 255)
GOLD = (255, 209, 102, 255)
TEXT = (247, 242, 255, 255)


def _png_chunk(tag: bytes, data: bytes) -> bytes:
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)


def write_png(path: Path, size: int) -> None:
    """Escribe un PNG RGBA sin dependencias externas."""
    pixels = bytearray()
    cx, cy, r = size // 2, size // 2, int(size * 0.36)

    for y in range(size):
        pixels.append(0)  # filtro PNG: ninguno
        for x in range(size):
            dx, dy = x - cx, y - cy
            dist = (dx * dx + dy * dy) ** 0.5
            if dist <= r * 0.55:
                color = GOLD
            elif dist <= r * 0.85:
                color = RED
            elif dist <= r:
                color = PURPLE
            else:
                color = BG
            # Letra A estilizada en el centro
            if size >= 32 and abs(dx) < r * 0.35 and dy < r * 0.15 and dy > -r * 0.55:
                if dy < -r * 0.05 or abs(dx) < r * 0.12:
                    color = TEXT if dist < r * 0.9 else color
            pixels.extend(color)

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    compressed = zlib.compress(bytes(pixels), 9)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + _png_chunk(b"IHDR", ihdr)
        + _png_chunk(b"IDAT", compressed)
        + _png_chunk(b"IEND", b"")
    )
    path.write_bytes(png)


def write_ico(path: Path, sizes: list[int]) -> None:
    """Empaqueta varios PNG incrustados en un ICO."""
    images: list[tuple[int, bytes]] = []
    for size in sizes:
        cx, cy, r = size // 2, size // 2, max(2, int(size * 0.36))
        and_mask = bytearray(size * ((size + 31) // 32) * 4)
        bmp = bytearray()
        for y in range(size - 1, -1, -1):
            bmp.extend(b"\x00")
            for x in range(size):
                dx, dy = x - cx, y - cy
                dist = (dx * dx + dy * dy) ** 0.5
                if dist <= r * 0.55:
                    b, g, rch, a = GOLD
                elif dist <= r * 0.85:
                    b, g, rch, a = RED
                elif dist <= r:
                    b, g, rch, a = PURPLE
                else:
                    b, g, rch, a = BG
                if size >= 16 and abs(dx) < r * 0.35 and dy < r * 0.15 and dy > -r * 0.55:
                    if dy < -r * 0.05 or abs(dx) < r * 0.12:
                        if dist < r * 0.9:
                            b, g, rch, a = TEXT
                bmp.extend(bytes([b, g, rch, a]))

        header = struct.pack("<IIIHHIIIIII", 40, size, size * 2, 1, 32, 0, len(bmp), 0, 0, 0, 0)
        images.append((size, header + bytes(bmp)))

    offset = 6 + 16 * len(images)
    parts = [struct.pack("<HHH", 0, 1, len(images))]
    data_parts = []
    for size, blob in images:
        w = 0 if size == 256 else size
        h = 0 if size == 256 else size
        parts.append(struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(blob), offset))
        data_parts.append(blob)
        offset += len(blob)
    path.write_bytes(b"".join(parts) + b"".join(data_parts))


def main() -> None:
    try:
        OUT_DIR.mkdir(parents=True, exist_ok=True)
        write_png(PNG_PATH, 256)
        write_ico(ICO_PATH, [16, 32, 48, 256])
        print(f"[SUCCESS] {PNG_PATH}")
        print(f"[SUCCESS] {ICO_PATH}")
    except OSError as exc:
        print(f"[ERROR] {exc}")
        raise SystemExit(1) from exc


if __name__ == "__main__":
    main()
