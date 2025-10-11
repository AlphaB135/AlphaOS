# Building OVMF for AegisOS fxig08ouh

This project assumes an OVMF firmware build compatible with QEMU's `q35` machine. Use the official edk2 repository.

```bash
sudo apt-get install build-essential uuid-dev nasm python3-distutils
mkdir -p ~/src && cd ~/src
git clone https://github.com/tianocore/edk2.git --depth=1
cd edk2
./edksetup.sh
make -C BaseTools
source edksetup.sh
build -a X64 -t GCC5 -p OvmfPkg/OvmfPkgX64.dsc -b RELEASE
```

Artifacts appear under `Build/OvmfX64/RELEASE_GCC5/`. Two files are required:

- `OVMF_CODE.fd`: read-only firmware image supplied to `-drive if=pflash,readonly=on`
- `OVMF_VARS.fd`: mutable NVRAM image copied alongside `OVMF_CODE.fd`

The run script (`scripts/run-qemu.sh`) expects both files in the same directory. Supply the path via `--ovmf`:

```bash
./scripts/run-qemu.sh --ovmf ~/src/edk2/Build/OvmfX64/RELEASE_GCC5/FV/OVMF_CODE.fd
```

For faster iteration, consider installing the `ovmf` package provided by your distribution.
