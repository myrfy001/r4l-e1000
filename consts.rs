
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
pub(crate) const E1000_RA:usize = 0x05400;	/* Receive Address - RW Array */
pub(crate) const E1000_MTA:usize = 0x05200 ;	/* Multicast Table Array - RW Array */

pub(crate) const E1000_RDH:usize = 0x02810;	/* RX Descriptor Head - RW */
pub(crate) const E1000_RDT:usize = 0x02818;	/* RX Descriptor Tail - RW */
pub(crate) const E1000_RDLEN:usize = 0x02808;	/* RX Descriptor Length - RW */
pub(crate) const E1000_RDBAL:usize = 0x02800;	/* RX Descriptor Base Address Low - RW */
pub(crate) const E1000_RDBAH:usize = 0x02804;	/* RX Descriptor Base Address High - RW */
pub(crate) const E1000_TDH:usize = 0x03810;	/* TX Descriptor Head - RW */
pub(crate) const E1000_TDT:usize = 0x03818;	/* TX Descripotr Tail - RW */
pub(crate) const E1000_TDLEN:usize = 0x03808;	/* TX Descriptor Length - RW */
pub(crate) const E1000_TDBAL:usize = 0x03800;	/* TX Descriptor Base Address Low - RW */
pub(crate) const E1000_TDBAH:usize = 0x03804;	/* TX Descriptor Base Address High - RW */


// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
// pub(crate) const E1000_:usize = ;	/*  */
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


