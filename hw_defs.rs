
// Defined in intel chip manual section 3.3.3
#[repr(C)]
pub(crate) struct TxDescEntry {
    pub(crate) buf_addr: u64,
    pub(crate) length: u16,
    pub(crate) cso: u8,
    pub(crate) cmd: u8,
    pub(crate) sta: u8,
    pub(crate) css: u8,
    pub(crate) special: u16,
}


// Defined in intel chip manual section 3.2.3
#[repr(C)]
pub(crate) struct RxDescEntry {
    pub(crate) buf_addr: u64,
    pub(crate) length: u16,
    pub(crate) checksum: u16,
    pub(crate) status: u8,
    pub(crate) errors: u8,
    pub(crate) special: u16,
}

