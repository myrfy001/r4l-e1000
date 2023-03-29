// SPDX-License-Identifier: GPL-2.0

//! Rust for linux e1000 driver demo

#![allow(unused)]

use core::iter::Iterator;
use core::sync::atomic::AtomicPtr;

use kernel::pci::Resource;
use kernel::prelude::*;
use kernel::sync::Arc;
use kernel::{pci, device, driver, bindings, net, dma, c_str};
use kernel::device::RawDevice;
use kernel::sync::SpinLock;


use hw_defs::{RxRingBuf, TxRingBuf, TxDescEntry, RxDescEntry};
mod consts;
mod hw_defs;
mod e1000_ops;

use e1000_ops::E1000Ops;

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
    e1000_hw_ops: E1000Ops,
    tx_ring: SpinLock<Option<TxRingBuf>>,
    rx_ring: SpinLock<Option<RxRingBuf>>,
    irq: u32,
    _irq_handler: AtomicPtr<kernel::irq::Registration<E1000InterruptHandler>>
}


// TODO not sure why it is safe to do this.
unsafe impl Send for NetDevicePrvData {}
unsafe impl Sync for NetDevicePrvData {}

struct NetDevice {}


impl NetDevice {
    fn e1000_setup_all_tx_resources(data: &NetDevicePrvData) -> Result<TxRingBuf> {

        // Alloc dma memory space for tx desciptors
        let dma_desc = dma::Allocation::<hw_defs::TxDescEntry>::try_new(&*data.dev, TX_RING_SIZE, bindings::GFP_KERNEL)?;
        
        // Safety: all fields of the slice members will be inited below.
        let tx_ring = unsafe{core::slice::from_raw_parts_mut(dma_desc.cpu_addr, TX_RING_SIZE)};
        
        // Alloc dma memory space for buffers
        let dma_buf = dma::Allocation::<u8>::try_new(&*data.dev, TX_RING_SIZE * RXTX_SINGLE_RING_BLOCK_SIZE, bindings::GFP_KERNEL)?;
        
        tx_ring.iter_mut().enumerate().for_each(|(idx, desc)| {
            desc.buf_addr = (dma_buf.dma_handle as usize + RXTX_SINGLE_RING_BLOCK_SIZE * idx) as u64;
            desc.cmd = 0;
            desc.length = 0;
            desc.cso = 0;
            desc.css = 0;
            desc.special = 0;
            desc.sta = E1000_TXD_STAT_DD as u8;  // Mark all the descriptors as Done, so the first packet can be transmitted.
        });
        Ok(TxRingBuf::new(dma_desc, dma_buf, TX_RING_SIZE, RXTX_SINGLE_RING_BLOCK_SIZE))
    }

    fn e1000_setup_all_rx_resources(data: &NetDevicePrvData) -> Result<RxRingBuf> {

        // Alloc dma memory space for rx desciptors
        let dma_desc = dma::Allocation::<hw_defs::RxDescEntry>::try_new(&*data.dev, RX_RING_SIZE, bindings::GFP_KERNEL)?;
        
        // Safety: all fields of the slice members will be inited below.
        let rx_ring = unsafe{core::slice::from_raw_parts_mut(dma_desc.cpu_addr, RX_RING_SIZE)};
        
        // Alloc dma memory space for buffers
        let dma_buf = dma::Allocation::<u8>::try_new(&*data.dev, RX_RING_SIZE * RXTX_SINGLE_RING_BLOCK_SIZE, bindings::GFP_KERNEL)?;
        
        rx_ring.iter_mut().enumerate().for_each(|(idx, desc)| {
            desc.buf_addr = (dma_buf.dma_handle as usize + RXTX_SINGLE_RING_BLOCK_SIZE * idx) as u64;
            desc.length = 0;
            desc.special = 0;
            desc.checksum = 0;
            desc.status = 0;
            desc.errors = 0;
        });


        Ok(RxRingBuf::new(dma_desc, dma_buf, RX_RING_SIZE, RXTX_SINGLE_RING_BLOCK_SIZE))
    }


}

#[vtable]
impl net::DeviceOperations for NetDevice {
    
    type Data = Box<NetDevicePrvData>;

    fn open(dev: &net::Device, data: &NetDevicePrvData) -> Result {
        pr_info!("Rust for linux e1000 driver demo (net device open)\n");

        dev.netif_carrier_off();

        // init dma memory for tx and rx
        let tx_ringbuf = Self::e1000_setup_all_tx_resources(data)?;
        let rx_ringbuf = Self::e1000_setup_all_rx_resources(data)?;

        // TODO e1000_power_up_phy() not implemented. It's used in case of PHY *MAY* power down,
        // which will not be supported in this MVP driver.
        

        // modify e1000's hardware registers, give rx/tx queue info to the nic.
        data.e1000_hw_ops.e1000_configure(&rx_ringbuf, &tx_ringbuf)?;

        *data.rx_ring.lock_irqdisable() = Some(rx_ringbuf);
        *data.tx_ring.lock_irqdisable() = Some(tx_ringbuf);

        let irq_prv_data = Box::try_new(IrqPrivateData{

        })?;
        
        // Again, the `irq::Registration` contains an `irq::InternalRegistration` which implemented `Drop`, so 
        // we mustn't let it dropped.
        // TODO: there is memory leak now. 
        let req_reg = kernel::irq::Registration::<E1000InterruptHandler>::try_new(data.irq, irq_prv_data, kernel::irq::flags::SHARED, fmt!("{}",data.dev.name()))?;
        data._irq_handler.store(Box::into_raw(Box::try_new(req_reg)?), core::sync::atomic::Ordering::Relaxed);

        data.napi.enable();

        dev.netif_start_queue();

        dev.netif_carrier_on();

        Ok(())
    }

    fn stop(_dev: &net::Device, _data: &NetDevicePrvData) -> Result {
        pr_info!("Rust for linux e1000 driver demo (net device stop)\n");
        Ok(())
    }

    fn start_xmit(skb: &net::SkBuff, dev: &net::Device, data: &NetDevicePrvData) -> net::NetdevTx {

        if skb.head_data().len() > RXTX_SINGLE_RING_BLOCK_SIZE {
            pr_err!("xmit msg too long");
            return net::NetdevTx::Busy
        }

        let tx_ring = data.tx_ring.lock_irqdisable();
        let mut tdt = data.e1000_hw_ops.e1000_read_tx_queue_tail();
        let mut tdh = data.e1000_hw_ops.e1000_read_tx_queue_head();
        let mut rdt = data.e1000_hw_ops.e1000_read_rx_queue_tail();
        let mut rdh = data.e1000_hw_ops.e1000_read_rx_queue_head();

        pr_info!("Rust for linux e1000 driver demo (net device start_xmit) tdt={}, tdh={}, rdt={}, rdh={}\n", tdt, tdh, rdt, rdh);
        pr_info!("Interrupt State: {:x}", data.e1000_hw_ops.e1000_read_interrupt_state());

        /* On PCI/PCI-X HW, if packet size is less than ETH_ZLEN,
        * packets may get corrupted during padding by HW.
        * To WA this issue, pad all small packets manually.
        */
        skb.put_padto(bindings::ETH_ZLEN);
        
        let tx_ring = tx_ring.as_ref().unwrap();
        let tx_descs:&mut [TxDescEntry] = tx_ring.as_desc_slice();
        let tx_desc = &mut tx_descs[tdt as usize];
        if tx_desc.sta & E1000_TXD_STAT_DD as u8 == 0 {
            pr_err!("xmit busy");
            return net::NetdevTx::Busy
        }

        let tx_buf: &mut [u8] = tx_ring.as_buf_slice(tdt as usize);
        let src = skb.head_data();
        tx_buf[..src.len()].copy_from_slice(src);
        tx_desc.length = src.len() as u16;
        tx_desc.cmd = ((E1000_TXD_CMD_RS | E1000_TXD_CMD_EOP) >> 24) as u8;
        tx_desc.sta = 0;

        // TODO memory fence here. we are testing it on an x86, so maybe left it out is ok.

        tdt = (tdt + 1) % TX_RING_SIZE as u32;
        data.e1000_hw_ops.e1000_write_tx_queue_tail(tdt);

        dev.sent_queue(skb.len());

        
        
        skb.napi_consume(64);
        dev.completed_queue(1, skb.len());
        
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

// since the ownership limitation, We can't use NetDevicePrvData as C code, so we need to define a new type here. 
struct IrqPrivateData {}

struct E1000InterruptHandler {}

impl kernel::irq::Handler for E1000InterruptHandler {
    type Data = Box<IrqPrivateData>;

    fn handle_irq(data: &IrqPrivateData) -> kernel::irq::Return {
        pr_info!("Rust for linux e1000 driver demo (handle_irq)\n");
        kernel::irq::Return::Handled
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
        pr_info!("Rust for linux e1000 driver demo (napi poll)\n");
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

        // Note: only support QEMU's 82540EM chip now.
        
        // this works like a filter, the PCI device may have up to 6 bars, those bars have different types,
        // some of them are mmio, others are io-port based. The params to the following function is a 
        // filter condition, and the return value is a mask indicating which of those bars are selected.
        let bars = dev.select_bars((bindings::IORESOURCE_MEM | bindings::IORESOURCE_IO) as u64);

        // the underlying will call `pci_enable_device()`. the R4L framework doesn't support `pci_enable_device_memory()` now.
        dev.enable_device()?;

        // ask the os to reserve the physical memory region of the selected bars.
        dev.request_selected_regions(bars, c_str!("e1000 reserved memory"))?;

        // set device to master mode.
        dev.set_master();

        // get resource(memory range) provided by BAR0
        let mem_res = dev.iter_resource().nth(0).ok_or(kernel::error::code::EIO)?;
        let io_res = dev.iter_resource().skip(1).find(|r:&Resource|r.check_flags(bindings::IORESOURCE_IO)).ok_or(kernel::error::code::EIO)?;

        // TODO pci_save_state(pdev); not supported by crate now, only have raw C bindings.

        // alloc new ethernet device, this line represent the `alloc_etherdev()` and `SET_NETDEV_DEV()` in C version.
        let mut netdev_reg = net::Registration::<NetDevice>::try_new(dev)?;
        let netdev = netdev_reg.dev_get();

        // map device registers' hardware address to logical address so the kernel driver can access it.
        let mem_addr = Arc::try_new(dev.map_resource(&mem_res, mem_res.len())?)?;

        // get the io-port based address
        let io_addr = Arc::try_new(pci::IoPort::try_new(&io_res)?)?;



        // TODO implement C version `e1000_init_hw_struct()`

        // only pci-x need 64-bit, to simplify code, hardcode 32-bit for now.
        dma::set_coherent_mask(dev, 0xFFFFFFFF)?;

        // TODO ethtool support here.

        // Enable napi, the R4L will call `netif_napi_add_weight()`, the origin C version calls `netif_napi_add`
        let napi = net::NapiAdapter::<NAPI>::add_weight(&netdev, 64)?;


        // TODO implement C version `e1000_sw_init()`

        // TODO a lot of feature flags are assigned here in the C code, skip them for now.
        let e1000_hw_ops = E1000Ops {
            mem_addr: Arc::clone(&mem_addr),
            io_addr: Arc::clone(&io_addr),
        };
        e1000_hw_ops.e1000_reset_hw()?;


        // TODO: the MAC address is hardcoded here, should be read out from EEPROM later.
        netdev.eth_hw_addr_set(&MAC_HWADDR);

        // TODO: Some background tasks and Wake on LAN are not supported now.

        let irq = dev.irq();

        let common_dev = device::Device::from_dev(dev);

        netdev.netif_carrier_off();


        // SAFETY: `spinlock_init` is called below.
        let mut tx_ring = unsafe{SpinLock::new(None)};
        let mut rx_ring = unsafe{SpinLock::new(None)};
        // SAFETY: We don't move `tx_ring` and `rx_ring`.
        kernel::spinlock_init!(unsafe{Pin::new_unchecked(&mut tx_ring)}, "tx_ring");
        kernel::spinlock_init!(unsafe{Pin::new_unchecked(&mut rx_ring)}, "rx_ring");


        netdev_reg.register(Box::try_new(
            NetDevicePrvData {
                dev: Arc::try_new(common_dev)?,
                e1000_hw_ops,
                napi: napi.into(),
                tx_ring,
                rx_ring,
                irq,
                _irq_handler: AtomicPtr::new(core::ptr::null_mut()),
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

/// pr
pub fn print_hex_dump(b:&[u8], l: usize) {
    for x in &b[..l] {
        pr_info!("{:x} ", x);
    }
}