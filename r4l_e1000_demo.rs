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

    napi: Arc<net::Napi>,
    hw_addr: Arc<pci::MappedResource>,
    irq: u32,
}


// TODO not sure why it is safe to do this.
unsafe impl Send for NetDevicePrvData {}
unsafe impl Sync for NetDevicePrvData {}

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

        // init dma memory for tx and rx
        Self::setup_tx_resource(data)?;
        Self::setup_rx_resource(data)?;

        let _ = data.irq;
        let _ = data.hw_addr;


        Ok(())
    }

    fn stop(_dev: &net::Device, _data: <Self::Data as kernel::PointerWrapper>::Borrowed<'_>) -> Result {
        pr_info!("Rust for linux e1000 driver demo (net device stop)\n");
        Ok(())
    }

    fn start_xmit(_skb: &net::SkBuff, _dev: &net::Device, _data: <Self::Data as kernel::PointerWrapper>::Borrowed<'_>,) -> net::NetdevTx {
        pr_info!("Rust for linux e1000 driver demo (net device start_xmit)\n");
        net::NetdevTx::Ok
    }

    fn get_stats64(_netdev: &net::Device, _data: &NetDevicePrvData, stats: &mut net::RtnlLinkStats64) {
        pr_info!("Rust for linux e1000 driver demo (net device get_stats64)\n");
        
        // stats.set_rx_bytes(data.stats.rx_bytes.load(Ordering::Relaxed));
        // stats.set_rx_packets(data.stats.rx_packets.load(Ordering::Relaxed));
        // stats.set_tx_bytes(data.stats.tx_bytes.load(Ordering::Relaxed));
        // stats.set_tx_packets(data.stats.tx_packets.load(Ordering::Relaxed));

        stats.set_rx_bytes(0);
        stats.set_rx_packets(0);
        stats.set_tx_bytes(0);
        stats.set_tx_packets(0);
    }
}

/// the private data for the adapter
struct E1000DrvPrvData {
    _netdev_reg: net::Registration<NetDevice>,
}

impl driver::DeviceRemoval for E1000DrvPrvData {
    fn device_remove(&self) {
        pr_info!("Rust for linux e1000 driver demo (device_remove)\n");
    }
}

struct NAPI{}

impl net::NapiPoller for NAPI {
    type Data = Box<NetDevicePrvData>;

    fn poll(
        _napi: &net::Napi,
        _budget: i32,
        _dev: &net::Device,
        _data: &NetDevicePrvData,
    ) -> i32 {
        let _ = _data.napi;
        todo!()
    }
}

struct E1000Drv {}

impl pci::Driver for E1000Drv {

    // The Box type has implemented PointerWrapper trait.
    type Data = Box<E1000DrvPrvData>;

    kernel::define_pci_id_table! {(), [
        (pci::DeviceId::new(E1000_VENDER_ID, E1000_DEVICE_ID), None),
    ]}


    fn probe(dev: &mut pci::Device, id: core::option::Option<&Self::IdInfo>) -> Result<Self::Data> {
        pr_info!("Rust for linux e1000 driver demo (probe): {:?}\n", id);

        
        // this works like a filter, the PCI device may have up to 6 bars, those bars have different types,
        // some of them are mmio, others are io-port based. The params to the following function is a 
        // filter condition, and the return value is a mask indicating which of those bars are selected.
        let bars = dev.select_bars(bindings::IORESOURCE_MEM as u64);

        // the underlying will call `pci_enable_device()`, but the C version use `pci_enable_device_mem()`
        // TODO: find the difference between the two
        dev.enable_device()?;

        // ask the os to reserve the memory region of the selected bars.
        dev.request_selected_regions(bars, c_str!("e1000 reserved memory"))?;

        // set device to master mode.
        dev.set_master();

        // get resource(memory range) provided by BAR0
        let res = dev.iter_resource().nth(0).ok_or(kernel::error::code::EIO)?;

        // TODO pci_save_state(pdev); not supported by crate now, only have raw C bindings.

        // alloc new ethernet device, this line represent the `alloc_etherdev()` and `SET_NETDEV_DEV()` in C version.
        let mut netdev_reg = net::Registration::<NetDevice>::try_new(dev)?;
        let netdev = netdev_reg.dev_get();

        // map device registers' hardware address to logical address so the kernel driver can access it.
        let hw_addr = Arc::try_new(dev.map_resource(&res, res.len())?)?;

        // TODO implement C version `e1000_init_hw_struct()`

        // only pci-x need 64-bit, to simplify code, hardcode 32-bit for now.
        dma::set_coherent_mask(dev, 0xFFFFFFFF)?;

        // TODO ethtool support here.

        // Enable napi, the R4L will call `netif_napi_add_weight()`, the origin C version calls `netif_napi_add`
        let napi = net::NapiAdapter::<NAPI>::add_weight(&netdev, 64)?;


        // TODO implement C version `e1000_sw_init()`

        // TODO a lot of feature flags are assigned here in the C code, skip them for now.





        let irq = dev.irq();

        


        
        
        netdev.eth_hw_addr_set(&MAC_HWADDR);


        let common_dev = device::Device::from_dev(dev);

        
        netdev.netif_carrier_off();

        
        netdev_reg.register(Box::try_new(
            NetDevicePrvData {
                dev: Arc::try_new(common_dev)?,
                hw_addr: Arc::clone(&hw_addr),
                napi: napi.into(),
                irq,
            }
        )?)?;

        

        

        
        

        Ok(Box::try_new(
            E1000DrvPrvData{
                // Must hold this registration, or the device will be removed.
                _netdev_reg: netdev_reg,
            }
        )?)
    }

    fn remove(data: &Self::Data) {
        pr_info!("Rust for linux e1000 driver demo (remove)\n");
        drop(data);
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