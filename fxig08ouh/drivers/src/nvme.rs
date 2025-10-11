//! NVMe controller skeleton exposing Identify and Read primitives.

#[derive(Debug)]
pub enum NvmeError {
    ControllerMissing,
    IoError,
}

#[derive(Clone, Debug)]
pub struct IdentifyController {
    pub vendor_id: u16,
    pub model_number: [u8; 40],
    pub serial_number: [u8; 20],
}

pub fn init() {
    // TODO: enumerate PCIe bus for NVMe controller and map BARs.
}

pub fn identify_controller() -> Result<IdentifyController, NvmeError> {
    Ok(IdentifyController {
        vendor_id: 0x1234,
        model_number: *b"FXIG08OUH NVME MODEL.............",
        serial_number: *b"FXIG08OUHNVME0000",
    })
}

pub fn log_identify(data: &IdentifyController) {
    log::info!(
        "NVMe: vendor={:#x} model={:?}",
        data.vendor_id,
        core::str::from_utf8(&data.model_number).unwrap_or("<utf8>")
    );
}

pub fn read_block(_lba: u64, buffer: &mut [u8]) -> Result<(), NvmeError> {
    for (idx, byte) in buffer.iter_mut().enumerate() {
        *byte = (idx & 0xFF) as u8;
    }
    Ok(())
}

pub fn log_sector(buffer: &[u8]) {
    log::info!("NVMe read ({:02X?} ...)", &buffer[..core::cmp::min(buffer.len(), 16)]);
}
