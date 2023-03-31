use kernel::prelude::*;
use kernel::pci::{MappedResource, IoPort};
use kernel::delay::coarse_sleep;
use kernel::sync::Arc;

use core::time::Duration;

use crate::ring_buf::{RxRingBuf, TxRingBuf};

use crate::consts::*;

pub(crate) struct E1000Ops {
    pub(crate) mem_addr: Arc<MappedResource>,
    pub(crate) io_addr: Arc<IoPort>,
}

impl E1000Ops {

    /// reset the hardware completely, correspond to C version `e1000_reset_hw`.
    /// only add support for QEMU's 82540EM chip.
    pub(crate) fn e1000_reset_hw(&self) -> Result{

        /* Clear interrupt mask to stop board from generating interrupts */
        self.mem_addr.writel(0xffffffff, E1000_IMC)?;

        /* Disable the Transmit and Receive units.  Then delay to allow
        * any pending transactions to complete before we hit the MAC with
        * the global reset.
        */
        self.mem_addr.writel(0, E1000_RCTL)?;
        self.mem_addr.writel(E1000_TCTL_PSP, E1000_TCTL)?;
        self.e1000_write_flush();

        /* Delay to allow any outstanding PCI transactions to complete before
         * resetting the device
         */
        coarse_sleep(Duration::from_millis(10));

        let ctrl = self.mem_addr.readl(E1000_CTRL)?;

        /* These controllers can't ack the 64-bit write when issuing the
		 * reset, so use IO-mapping as a workaround to issue the reset
		 */
        self.e1000_write_reg_io(ctrl | E1000_CTRL_RST, E1000_CTRL)?;

        /* After MAC reset, force reload of EEPROM to restore power-on settings
         * to device.  Later controllers reload the EEPROM automatically, so
         * just wait for reload to complete.
         * Auto read done will delay 5ms.
         */
        coarse_sleep(Duration::from_millis(5));

        /* Disable HW ARPs on ASF enabled adapters */
        let manc = self.mem_addr.readl(E1000_MANC)?;
        self.mem_addr.writel(manc & (!E1000_MANC_ARP_EN), E1000_MANC)?;

        /* Clear interrupt mask to stop board from generating interrupts */
        self.mem_addr.writel(0xffffffff, E1000_IMC)?;
        
        /* Clear any pending interrupt events. */
        self.mem_addr.readl(E1000_ICR)?;

        Ok(())
    }

    fn e1000_write_flush(&self){
        // This read shouldn't fail 
        self.mem_addr.readl(E1000_STATUS).unwrap();
    }

    fn e1000_write_reg_io(&self, value: u32, addr: usize) -> Result {
        self.io_addr.outl(addr as u32, 0)?;
        self.io_addr.outl(value, 4)?;
        Ok(())
    }

    pub(crate) fn e1000_configure(&self, rx_ring: &RxRingBuf, tx_ring: &TxRingBuf) -> Result {
        self.e1000_configure_rx(rx_ring)?;
        self.e1000_configure_tx(tx_ring)?;

        // Enable related interrupts
        self.mem_addr.writel(
            E1000_ICR_TXDW | E1000_ICR_RXT0 | E1000_ICR_RXDMT0 | E1000_ICR_RXSEQ | E1000_ICR_LSC,
            E1000_IMS
        )?;
        Ok(())
    }



    fn e1000_configure_tx(&self, tx_ring: &TxRingBuf) -> Result {
        // According to Manual 14.5

        // set ring buf head index, tail index and buf size
        self.mem_addr.writel(0, E1000_TDH)?;
        self.mem_addr.writel(0, E1000_TDT)?;
        self.mem_addr.writel((TX_RING_SIZE * 16) as u32, E1000_TDLEN)?;
        // set ring buf start address
        self.mem_addr.writel(tx_ring.desc.get_dma_addr() as u32, E1000_TDBAL)?;
        self.mem_addr.writel(0, E1000_TDBAH)?;

        let tctl = (
            E1000_TCTL_EN | 
            E1000_TCTL_PSP |
            0x10 << E1000_CT_SHIFT | 
            0x40 << E1000_COLD_SHIFT
        );
        self.mem_addr.writel(tctl, E1000_TCTL)?;

        let tipg = (
            DEFAULT_82543_TIPG_IPGT_COPPER | 
            DEFAULT_82543_TIPG_IPGR1 << E1000_TIPG_IPGR1_SHIFT |
            DEFAULT_82543_TIPG_IPGR2 << E1000_TIPG_IPGR2_SHIFT
        );
        self.mem_addr.writel(tipg, E1000_TIPG)?;
        

        Ok(())

    }

    // fn e1000_setup_rctl(&self) {

    // }

    fn e1000_configure_rx(&self, rx_ring: &RxRingBuf) -> Result {
        // According to Manual 14.4

        // According to MIT6.828 Exercise 10, hardcode to QEMU's MAC address.
        // 52:54:00:12:34:56
        self.mem_addr.writel(0x12005452, E1000_RA)?;      //RAL
        self.mem_addr.writel(0x5634 | (1 << 31), E1000_RA + 4)?;  //RAH

        for i in 0..128 {
            self.mem_addr.writel(0, E1000_MTA + i * 4)?;
        }

        
        self.mem_addr.writel(0, E1000_RDH)?;
        self.mem_addr.writel((RX_RING_SIZE - 1) as u32, E1000_RDT)?;
        self.mem_addr.writel((RX_RING_SIZE * 16) as u32, E1000_RDLEN)?;
        self.mem_addr.writel(rx_ring.desc.get_dma_addr() as u32, E1000_RDBAL)?;
        self.mem_addr.writel(0, E1000_RDBAH)?;

        let rctl = (
            E1000_RCTL_EN | 
            E1000_RCTL_BAM | 
            E1000_RCTL_SZ_2048 | 
            E1000_RCTL_SECRC
        );
        self.mem_addr.writel(rctl, E1000_RCTL)?;

        // Disable RDTR and RADV timer, since we use NAPI, we don't need hardware to help us decrease interrupts.
        self.mem_addr.writel(0, E1000_RDTR)?;
        self.mem_addr.writel(0, E1000_RADV)?;
        
        Ok(())
    }

    pub(crate) fn e1000_read_interrupt_state(&self) -> u32 {
        self.mem_addr.readl(E1000_ICR).unwrap()
    }

    pub(crate) fn e1000_read_tx_queue_head(&self) -> u32 {
        self.mem_addr.readl(E1000_TDH).unwrap()
    }

    pub(crate) fn e1000_read_tx_queue_tail(&self) -> u32 {
        self.mem_addr.readl(E1000_TDT).unwrap()
    }

    pub(crate) fn e1000_write_tx_queue_tail(&self, val: u32) {
        self.mem_addr.writel(val, E1000_TDT).unwrap()
    }


    pub(crate) fn e1000_read_rx_queue_head(&self) -> u32 {
        self.mem_addr.readl(E1000_RDH).unwrap()
    }

    pub(crate) fn e1000_read_rx_queue_tail(&self) -> u32 {
        self.mem_addr.readl(E1000_RDT).unwrap()
    }

    pub(crate) fn e1000_write_rx_queue_tail(&self, val: u32) {
        self.mem_addr.writel(val, E1000_RDT).unwrap()
    }


}

