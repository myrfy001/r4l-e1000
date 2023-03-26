// SPDX-License-Identifier: GPL-2.0

//! Rust for linux e1000 driver demo

use kernel::prelude::*;

module! {
    type: E1000KernelMod,
    name: "r4l_e1000_demo",
    author: "Myrfy001",
    description: "Rust for linux e1000 driver demo",
    license: "GPL",
}

struct E1000KernelMod {}

impl kernel::Module for E1000KernelMod {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust for linux e1000 driver demo (init)\n");

        Ok(E1000KernelMod {})
    }
}

impl Drop for E1000KernelMod {
    fn drop(&mut self) {
        pr_info!("Rust for linux e1000 driver demo (exit)\n");
    }
}