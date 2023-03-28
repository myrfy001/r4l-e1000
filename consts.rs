
pub(crate) const RX_RING_SIZE:usize = 256;
pub(crate) const TX_RING_SIZE:usize = 256;
pub(crate) const RXTX_SINGLE_RING_BLOCK_SIZE:usize = 8192;

pub(crate) const MAC_HWADDR: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x55];

pub(crate) const E1000_VENDER_ID:u32 = 0x8086;
pub(crate) const E1000_DEVICE_ID:u32 = 0x100E;


// E1000 Regs

pub(crate) const E1000_CTRL:usize = 0x00000;	/* Device Control - RW */
pub(crate) const E1000_STATUS:usize = 0x00008;	/* Device Status - RO */
pub(crate) const E1000_IMC:usize = 0x000D8;	/* Interrupt Mask Clear - WO */
pub(crate) const E1000_RCTL:usize = 0x00100;	/* RX Control - RW */
pub(crate) const E1000_TCTL:usize = 0x00400;	/* TX Control - RW */
pub(crate) const E1000_MANC:usize = 0x05820;	/* Management Control - RW */
pub(crate) const E1000_ICR:usize = 0x000C0;	/* Interrupt Cause Read - R/clr */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */



// E1000 Regs Fields
pub(crate) const E1000_TCTL_PSP:u32 = 0x00000008;	/* pad short packets */
pub(crate) const E1000_CTRL_RST:u32 = 0x04000000;	/* Global reset */
pub(crate) const E1000_MANC_ARP_EN:u32 = 0x00002000;	/* Enable ARP Request Filtering */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */




