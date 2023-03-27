// SPDX-License-Identifier: GPL-2.0

//! Rust for linux e1000 driver demo

use core::iter::Iterator;

use kernel::prelude::*;
use kernel::sync::Arc;
use kernel::{pci, device, driver, bindings, net, dma, c_str};

mod consts;
mod hw_defs;

use consts::*;


module! {
    type: E1000KernelMod,
    name: "r4l_e1000_demo",
    author: "Myrfy001",
    description: "Rust for linux e1000 driver demo",
    license: "GPL",
}


struct NetDevicePrvData {
    dev: Arc<device::Device>,
}

struct NetDevice {}


impl NetDevice {
    fn setup_tx_resource(data: &NetDevicePrvData) -> Result {

        // Alloc dma memory space for tx desciptors
        let dma_desc = dma::Allocation::<hw_defs::TxDescEntry>::try_new(&*data.dev, TX_RING_SIZE, bindings::GFP_KERNEL)?;
        
        // Safety: all fields of the slice members will be inited below.
        let tx_ring = unsafe{core::slice::from_raw_parts_mut(dma_desc.cpu_addr, TX_RING_SIZE)};
        
        // Alloc dma memory space for buffers
        let dma_buf = dma::Allocation::<u8>::try_new(&*data.dev, TX_RING_SIZE * RXTX_SINGLE_RING_BLOCK_SIZE, bindings::GFP_KERNEL)?;
        
        tx_ring.iter_mut().enumerate().for_each(|(idx, desc)| {
            desc.buf_addr = (dma_buf.cpu_addr as usize + RXTX_SINGLE_RING_BLOCK_SIZE * idx) as u64;
            desc.cmd = 0;
            desc.length = 0;
            desc.cso = 0;
            desc.css = 0;
            desc.special = 0;
            desc.sta = 0;
        });
        Ok(())
    }

    fn setup_rx_resource(data: &NetDevicePrvData) -> Result {

        // Alloc dma memory space for rx desciptors
        let dma_desc = dma::Allocation::<hw_defs::RxDescEntry>::try_new(&*data.dev, RX_RING_SIZE, bindings::GFP_KERNEL)?;
        
        // Safety: all fields of the slice members will be inited below.
        let rx_ring = unsafe{core::slice::from_raw_parts_mut(dma_desc.cpu_addr, RX_RING_SIZE)};
        
        // Alloc dma memory space for buffers
        let dma_buf = dma::Allocation::<u8>::try_new(&*data.dev, RX_RING_SIZE * RXTX_SINGLE_RING_BLOCK_SIZE, bindings::GFP_KERNEL)?;
        
        rx_ring.iter_mut().enumerate().for_each(|(idx, desc)| {
            desc.buf_addr = (dma_buf.cpu_addr as usize + RXTX_SINGLE_RING_BLOCK_SIZE * idx) as u64;
            desc.length = 0;
            desc.special = 0;
            desc.checksum = 0;
            desc.status = 0;
            desc.errors = 0;
        });
        Ok(())
    }


}

#[vtable]
impl net::DeviceOperations for NetDevice {
    
    type Data = Box<NetDevicePrvData>;

    fn open(_dev: &net::Device, data: <Self::Data as kernel::PointerWrapper>::Borrowed<'_>) -> Result {
        pr_info!("Rust for linux e1000 driver demo (net device open)\n");

        // init dma area
        
        Self::setup_tx_resource(data)?;
        Self::setup_rx_resource(data)?;

        todo!()
    }

    fn stop(_dev: &net::Device, _data: <Self::Data as kernel::PointerWrapper>::Borrowed<'_>) -> Result {
        pr_info!("Rust for linux e1000 driver demo (net device stop)\n");
        todo!()
    }

    fn start_xmit(_skb: &net::SkBuff, _dev: &net::Device, _data: <Self::Data as kernel::PointerWrapper>::Borrowed<'_>,) -> net::NetdevTx {
        pr_info!("Rust for linux e1000 driver demo (net device start_xmit)\n");
        todo!()
    }
}

/// the private data for the adapter
struct E1000DrvPrvData {

}

impl driver::DeviceRemoval for E1000DrvPrvData {
    fn device_remove(&self) {
        pr_info!("Rust for linux e1000 driver demo (device_remove)\n");
    }
}

struct E1000Drv {}

impl pci::Driver for E1000Drv {

    // The Box type has implemented PointerWrapper trait.
    type Data = Box<E1000DrvPrvData>;

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


        let common_dev = device::Device::from_dev(dev);

        let mut netdev: net::Registration<NetDevice> = net::Registration::<NetDevice>::try_new(dev)?;
        netdev.register(Box::try_new(
            NetDevicePrvData {
                dev: Arc::try_new(common_dev)?,
            }
        )?)?;

        dev.set_master();


        dma::set_coherent_mask(dev, 0xFFFFFFFF)?;

        // TODO pci_save_state(pdev); not supported by crate now, only have raw C bindings.

        // TODO alloc_etherdev
        
        

        Ok(Box::try_new(
            E1000DrvPrvData{}
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