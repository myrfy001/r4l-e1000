use kernel::dma;


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

pub(crate) struct RingBuf<T> {
    pub(crate) desc: dma::Allocation::<T>,
    pub(crate) buf: dma::Allocation::<u8>,
    len: usize,
    block_size: usize,
}

impl<T> RingBuf<T> {
    pub(crate) fn as_desc_slice(&self) -> &mut [T] {
        unsafe{core::slice::from_raw_parts_mut(self.desc.cpu_addr, self.len)}
    }

    pub(crate) fn as_buf_slice(&self, idx: usize) -> &mut [u8] {
        unsafe{core::slice::from_raw_parts_mut(self.buf.cpu_addr.offset((self.block_size * idx) as isize), self.block_size)}
    }

    pub(crate) fn new(desc: dma::Allocation::<T>, buf: dma::Allocation::<u8>, len: usize, block_size: usize) -> Self {
        Self {desc, buf, len, block_size}
    }
}

pub(crate) type RxRingBuf = RingBuf<RxDescEntry>;
pub(crate) type TxRingBuf = RingBuf<TxDescEntry>;