//! TPM measured boot helpers. Hashes boot metadata and attempts to extend the firmware log.

use core::ffi::c_void;

use mk::BootInfo;
use sha2::{Digest, Sha256};
use uefi::table::Boot;
use uefi::{Guid, Status};
use uefi::table::SystemTable;

const EFI_TCG2_PROTOCOL_GUID: Guid = Guid::from_fields(
    0x607f766c,
    0x7455,
    0x42be,
    0x93,
    0xf8,
    &[0x44, 0x4f, 0xc0, 0x3b, 0x13, 0x4d],
);

/// Measure boot information and attempt to extend the TPM event log.
pub fn measure_boot_info(st: &mut SystemTable<Boot>, info: &BootInfo) {
    let digest = digest_boot_info(info);
    if let Err(status) = extend_tpm_event(st, &digest) {
        log::warn!("TPM extend failed: {:?}", status);
    }
}

fn digest_boot_info(info: &BootInfo) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for region in info.memory_regions.iter() {
        hasher.update(region.base.to_le_bytes());
        hasher.update(region.length.to_le_bytes());
        hasher.update((region.kind as u32).to_le_bytes());
    }
    if let Some(fb) = info.framebuffer {
        hasher.update(fb.width.to_le_bytes());
        hasher.update(fb.height.to_le_bytes());
        hasher.update((fb.pixel_format as u32).to_le_bytes());
        hasher.update(fb.size.to_le_bytes());
        hasher.update(fb.phys_addr.to_le_bytes());
    }
    if let Some(rsdp) = info.rsdp_addr {
        hasher.update(rsdp.to_le_bytes());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&hasher.finalize());
    out
}

fn extend_tpm_event(st: &mut SystemTable<Boot>, digest: &[u8; 32]) -> Result<(), Status> {
    // SAFETY: UEFI guarantees protocol pointer validity while boot services live.
    let proto_ptr = unsafe { st.boot_services().locate_protocol_raw(&EFI_TCG2_PROTOCOL_GUID) }?;
    if proto_ptr.is_null() {
        return Err(Status::NOT_FOUND);
    }

    // TODO: cast to Tcg2Protocol once protocol bindings are wired; for now just log digest.
    log::info!("Measured boot digest: {:02x?}", &digest[..]);
    let _ = proto_ptr as *mut c_void;
    Ok(())
}
