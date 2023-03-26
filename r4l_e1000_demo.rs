// SPDX-License-Identifier: GPL-2.0

//! Rust for linux e1000 driver demo

use core::iter::Iterator;

use kernel::prelude::*;
use kernel::{pci, driver, bindings, c_str};

mod consts;

use consts::*;


module! {
    type: E1000KernelMod,
    name: "r4l_e1000_demo",
    author: "Myrfy001",
    description: "Rust for linux e1000 driver demo",
    license: "GPL",
}

/// the private data for the adapter
struct EtherDev {}

impl driver::DeviceRemoval for EtherDev {
    fn device_remove(&self) {
        pr_info!("Rust for linux e1000 driver demo (device_remove)\n");
    }
}

struct E1000Drv {}

impl pci::Driver for E1000Drv {

    // The Box type has implemented PointerWrapper trait.
    type Data = Box<EtherDev>;

    kernel::define_pci_id_table! {(), [
        (pci::DeviceId::new(E1000_VENDER_ID, E1000_DEVICE_ID), None),
    ]}


    fn probe(dev: &mut pci::Device, _id: core::option::Option<&Self::IdInfo>) -> Result<Self::Data> {
        pr_info!("Rust for linux e1000 driver demo (probe)\n");


        dev.enable_device()?;
        
        // this works like a filter, the PCI device may have up to 6 bars, those bars have different types,
        // some of them are mmio, others are io-port based. The params to the following function is a 
        // filter condition, and the return value is a mask indicating which of those bars are selected.
        let bars = dev.select_bars(bindings::IORESOURCE_MEM as u64);

        // ask the os to reserve the memory region of the selected bars.
        dev.request_selected_regions(bars, c_str!("e1000 reserved memory"))?;

        // get first resource
        let res = dev.iter_resource().nth(0).ok_or(kernel::error::code::EIO)?;

        // map device registers' hardware address to logical address so the kernel driver can access it.
        let reg_addr = dev.map_resource(&res, res.len())?;

        // test read the register, this test comes from https://pdos.csail.mit.edu/6.828/2011/labs/lab6/ Exercise 4.
        let v = reg_addr.readl(0x0008)?;
        pr_info!("read result: {}", v);
        


        
        dev.set_master();

        
        // TODO pci_save_state(pdev); not supported by crate now, only have raw C bindings.

        // TODO alloc_etherdev
        
        

        Ok(Box::try_new(
            EtherDev{}
        )?)
    }

    fn remove(_data: &Self::Data) {
        pr_info!("Rust for linux e1000 driver demo (remove)\n");
    }
}
struct E1000KernelMod {
    _dev: Pin<Box<driver::Registration::<pci::Adapter<E1000Drv>>>>,
}

impl kernel::Module for E1000KernelMod {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust for linux e1000 driver demo (init)\n");
        let d = driver::Registration::<pci::Adapter<E1000Drv>>::new_pinned(name, module)?;

        // we need to store `d` into the module struct, otherwise it will be dropped, which 
        // means the driver will be removed.
        Ok(E1000KernelMod {_dev: d})
    }
}

impl Drop for E1000KernelMod {
    fn drop(&mut self) {
        pr_info!("Rust for linux e1000 driver demo (exit)\n");
    }
}