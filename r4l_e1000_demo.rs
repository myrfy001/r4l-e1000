// SPDX-License-Identifier: GPL-2.0

//! Rust for linux e1000 driver demo

use kernel::prelude::*;
use kernel::{pci, driver};

mod consts;

use consts::*;


module! {
    type: E1000KernelMod,
    name: "r4l_e1000_demo",
    author: "Myrfy001",
    description: "Rust for linux e1000 driver demo",
    license: "GPL",
}

struct E1000Drv {}

impl pci::Driver for E1000Drv {
    kernel::define_pci_id_table! {(), [
        (pci::DeviceId::new(E1000_VENDER_ID, E1000_DEVICE_ID), None),
    ]}

    type Data = ();

    fn probe(_dev: &mut pci::Device, _id: core::option::Option<&Self::IdInfo>) -> Result<Self::Data> {
        pr_info!("Rust for linux e1000 driver demo (probe)\n");
        Ok(())
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