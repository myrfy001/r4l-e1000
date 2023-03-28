
pub(crate) const RX_RING_SIZE:usize = 256;
pub(crate) const TX_RING_SIZE:usize = 256;
pub(crate) const RXTX_SINGLE_RING_BLOCK_SIZE:usize = 2048;

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
pub(crate) const E1000_TIPG:usize = 0x00410;	/* TX Inter-packet gap -RW */


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
pub(crate) const E1000_TCTL_EN:u32 = 0x00000002;	/* enable tx */

pub(crate) const E1000_TCTL_CT:u32 = 0x00000ff0;	/* collision threshold */
pub(crate) const E1000_CT_SHIFT:u32 = 4;	

pub(crate) const E1000_TCTL_COLD:u32 = 0x003ff000;	/* collision distance */
pub(crate) const E1000_COLD_SHIFT:u32 = 12;	

/* Receive Control */
pub(crate) const E1000_RCTL_EN:u32 = 0x00000002;	/* enable */
pub(crate) const E1000_RCTL_BAM:u32 = 0x00008000;	/* broadcast enable */
pub(crate) const E1000_RCTL_SZ_2048:u32 = 0x00000000;	/* rx buffer size 2048 */
pub(crate) const E1000_RCTL_SECRC:u32 = 0x04000000;	/* Strip Ethernet CRC */

// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */


pub(crate) const E1000_CTRL_RST:u32 = 0x04000000;	/* Global reset */
pub(crate) const E1000_MANC_ARP_EN:u32 = 0x00002000;	/* Enable ARP Request Filtering */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */
// pub(crate) const E1000_:u32 = ;	/*  */



/* Default values for the transmit IPG register */
pub(crate) const DEFAULT_82543_TIPG_IPGT_COPPER:u32 = 8;
pub(crate) const DEFAULT_82543_TIPG_IPGR1:u32 = 8;
pub(crate) const E1000_TIPG_IPGR1_SHIFT:u32 = 10;
pub(crate) const DEFAULT_82543_TIPG_IPGR2:u32 = 6;
pub(crate) const E1000_TIPG_IPGR2_SHIFT:u32 = 20;

