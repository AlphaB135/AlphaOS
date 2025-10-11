#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 --ovmf <OVMF_CODE.fd> [--disk <disk.img>] [--headless] [--gdb] [--exit-pattern <string>]
Runs the fxig08ouh UEFI image inside QEMU with a virtio-net and NVMe device.
USAGE
}

OVMF_CODE=""
DISK_IMAGE="${DISK:-./target/disk.img}"
HEADLESS=0
WAIT_GDB=0
EXIT_PATTERN=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ovmf)
      OVMF_CODE="$2"
      shift 2
      ;;
    --disk)
      DISK_IMAGE="$2"
      shift 2
      ;;
    --headless)
      HEADLESS=1
      shift
      ;;
    --gdb)
      WAIT_GDB=1
      shift
      ;;
    --exit-pattern)
      EXIT_PATTERN="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "${OVMF_CODE}" ]]; then
  echo "--ovmf path is required" >&2
  exit 1
fi

OVMF_VARS="${OVMF_CODE%/*}/OVMF_VARS.fd"
if [[ ! -f "$OVMF_VARS" ]]; then
  cp "$OVMF_CODE" "$OVMF_VARS"
fi

SERIAL_PIPE=$(mktemp -u)
rm -f "$SERIAL_PIPE"
mkfifo "$SERIAL_PIPE"

cleanup() {
  rm -f "$SERIAL_PIPE"
}
trap cleanup EXIT

serial_monitor() {
  if [[ -n "$EXIT_PATTERN" ]]; then
    while IFS= read -r line; do
      echo "$line"
      if [[ "$line" == *"$EXIT_PATTERN"* ]]; then
        pkill -f "qemu-system-x86_64"
        break
      fi
    done < "$SERIAL_PIPE"
  else
    cat "$SERIAL_PIPE"
  fi
}

serial_monitor &

QEMU_ARGS=(
  -machine q35,accel=kvm:tcg
  -cpu max,+vmx,+svm
  -m 2048
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE"
  -drive if=pflash,format=raw,file="$OVMF_VARS"
  -drive id=nvme0,file="$DISK_IMAGE",format=raw,if=none
  -device nvme,drive=nvme0,serial=FXIG08OUHNVME
  -netdev user,id=net0
  -device virtio-net-pci,netdev=net0
  -device isa-debug-exit,iobase=0xf4,iosize=0x04
  -serial pipe:"$SERIAL_PIPE"
  -monitor stdio
)

if [[ $HEADLESS -eq 1 ]]; then
  QEMU_ARGS+=( -display none )
else
  QEMU_ARGS+=( -display sdl,gl=on )
fi

if [[ $WAIT_GDB -eq 1 ]]; then
  QEMU_ARGS+=( -s -S )
fi

exec qemu-system-x86_64 "${QEMU_ARGS[@]}"
