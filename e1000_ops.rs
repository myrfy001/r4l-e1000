use kernel::prelude::*;
use kernel::pci::{MappedResource, IoPort};
use kernel::delay::coarse_sleep;
use kernel::sync::Arc;

use core::time::Duration;

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
}