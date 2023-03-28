
pub(crate) const RX_RING_SIZE:usize = 256;
pub(crate) const TX_RING_SIZE:usize = 256;
pub(crate) const RXTX_SINGLE_RING_BLOCK_SIZE:usize = 8192;

pub(crate) const MAC_HWADDR: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x55];

pub(crate) const E1000_VENDER_ID:u32 = 0x8086;
pub(crate) const E1000_DEVICE_ID:u32 = 0x100E;