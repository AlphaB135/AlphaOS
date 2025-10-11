#!/usr/bin/env bash
set -euo pipefail

OUTPUT_IMAGE="${1:-target/fxig08ouh.iso}"
EFI_DIR="target/efi"
BOOT_DIR="$EFI_DIR/EFI/BOOT"
KERNEL_BIN="target/x86_64-unknown-uefi/release/boot.efi"

mkdir -p "$BOOT_DIR"

# Placeholder for EFI binary copy. Replace once build artifacts land here.
if [[ -f "$KERNEL_BIN" ]]; then
  cp "$KERNEL_BIN" "$BOOT_DIR/BOOTX64.EFI"
else
  printf 'WARN: EFI binary not found at %s\n' "$KERNEL_BIN" >&2
  printf '      Build the bootloader first: `just build`\n' >&2
  printf '      Creating empty placeholder image.\n' >&2
  printf 'UEFI placeholder %s\n' "$(date)" > "$BOOT_DIR/BOOTX64.EFI"
fi

genisoimage -quiet -V FXIG08OUH -o "$OUTPUT_IMAGE" -eltorito-alt-boot \
  -e EFI/BOOT/BOOTX64.EFI -no-emul-boot "$EFI_DIR"

echo "ISO created at $OUTPUT_IMAGE"
