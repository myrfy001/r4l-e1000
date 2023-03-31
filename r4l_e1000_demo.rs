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



mod consts;
mod hw_defs;
mod ring_buf;
mod e1000_ops;

use hw_defs::{TxDescEntry, RxDescEntry};
use ring_buf::{RxRingBuf, TxRingBuf};

use e1000_ops::E1000Ops;

use consts::*;


module! {
    type: E1000KernelMod,
    name: "r4l_e1000_demo",
    author: "Myrfy001",
    description: "Rust for linux e1000 driver demo",
    license: "GPL",
}


/// The private data for this driver
struct NetDevicePrvData {
    dev: Arc<device::Device>,
    napi: Arc<net::Napi>,
    e1000_hw_ops: Arc<E1000Ops>,
    tx_ring: SpinLock<Option<TxRingBuf>>,
    rx_ring: SpinLock<Option<RxRingBuf>>,
    irq: u32,
    _irq_handler: AtomicPtr<kernel::irq::Registration<E1000InterruptHandler>>
}


// TODO not sure why it is safe to do this.
unsafe impl Send for NetDevicePrvData {}
unsafe impl Sync for NetDevicePrvData {}

/// Represent the network device
struct NetDevice {}


impl NetDevice {

    /// Alloc the tx descriptor. But doesn't need to alloc buffer memory, since the network stack will pass in a SkBuff.
    fn e1000_setup_all_tx_resources(data: &NetDevicePrvData) -> Result<TxRingBuf> {

        // Alloc dma memory space for tx desciptors
        let dma_desc = dma::Allocation::<hw_defs::TxDescEntry>::try_new(&*data.dev, TX_RING_SIZE, bindings::GFP_KERNEL)?;
        
        // Safety: all fields of the slice members will be inited below.
        let tx_ring = unsafe{core::slice::from_raw_parts_mut(dma_desc.cpu_addr, TX_RING_SIZE)};
        
        
        tx_ring.iter_mut().enumerate().for_each(|(idx, desc)| {
            desc.buf_addr = 0;
            desc.cmd = 0;
            desc.length = 0;
            desc.cso = 0;
            desc.css = 0;
            desc.special = 0;
            desc.sta = E1000_TXD_STAT_DD as u8;  // Mark all the descriptors as Done, so the first packet can be transmitted.
        });
        Ok(TxRingBuf::new(dma_desc, TX_RING_SIZE))
    }


    /// Alloc the rx descriptor and the corresponding memory space. use `alloc_skb_ip_align` to alloc buffer and then map it to
    /// DMA address.
    fn e1000_setup_all_rx_resources(dev: &net::Device, data: &NetDevicePrvData) -> Result<RxRingBuf> {

        // Alloc dma memory space for rx desciptors
        let dma_desc = dma::Allocation::<hw_defs::RxDescEntry>::try_new(&*data.dev, RX_RING_SIZE, bindings::GFP_KERNEL)?;
        
        // Safety: all fields of the slice members will be inited below.
        let rx_ring_desc = unsafe{core::slice::from_raw_parts_mut(dma_desc.cpu_addr, RX_RING_SIZE)};
                
        // Alloc dma memory space for buffers
        let dma_buf = dma::Allocation::<u8>::try_new(&*data.dev, RX_RING_SIZE * RXTX_SINGLE_RING_BLOCK_SIZE, bindings::GFP_KERNEL)?;
        
        let mut rx_ring = RxRingBuf::new(dma_desc, RX_RING_SIZE);

        
        rx_ring_desc.iter_mut().enumerate().for_each(|(idx, desc)| {
            let skb = dev.alloc_skb_ip_align(RXTX_SINGLE_RING_BLOCK_SIZE as u32).unwrap();
            let dma_map = dma::MapSingle::try_new(&*data.dev, skb.head_data().as_ptr() as *mut u8, RXTX_SINGLE_RING_BLOCK_SIZE, bindings::dma_data_direction_DMA_FROM_DEVICE).unwrap();
            
            desc.buf_addr = dma_map.dma_handle as u64;
            desc.length = 0;
            desc.special = 0;
            desc.checksum = 0;
            desc.status = 0;
            desc.errors = 0;

            rx_ring.buf.borrow_mut()[idx] = Some((dma_map, skb));
        });

        Ok(rx_ring)
    }


    // corresponding to the C version e1000_clean_tx_irq()
    fn e1000_recycle_tx_queue(dev: &net::Device, data: &NetDevicePrvData) {
        let tdt = data.e1000_hw_ops.e1000_read_tx_queue_tail();
        let tdh = data.e1000_hw_ops.e1000_read_tx_queue_head();

        let mut tx_ring = data.tx_ring.lock_irqdisable();
        let mut tx_ring = tx_ring.as_mut().unwrap();

        let descs = tx_ring.desc.as_desc_slice();
        
        let mut idx = tx_ring.next_to_clean;
        while descs[idx].sta & E1000_TXD_STAT_DD as u8 != 0 && idx != tdh as usize {
            let (dm, skb) = tx_ring.buf.borrow_mut()[idx].take().unwrap();
            dev.completed_queue(1, skb.len());
            skb.napi_consume(64);
            drop(dm);
            drop(skb);

            idx = (idx + 1) % TX_RING_SIZE;
        }
        tx_ring.next_to_clean = idx;

    }


}

#[vtable]
impl net::DeviceOperations for NetDevice {
    
    type Data = Box<NetDevicePrvData>;

    /// this method will be called when you type `ip link set eth0 up` in your shell.
    fn open(dev: &net::Device, data: &NetDevicePrvData) -> Result {
        pr_info!("Rust for linux e1000 driver demo (net device open)\n");

        dev.netif_carrier_off();

        // init dma memory for tx and rx
        let tx_ringbuf = Self::e1000_setup_all_tx_resources(data)?;
        let rx_ringbuf = Self::e1000_setup_all_rx_resources(dev, data)?;

        // TODO e1000_power_up_phy() not implemented. It's used in case of PHY *MAY* power down,
        // which will not be supported in this MVP driver.
        

        // modify e1000's hardware registers, give rx/tx queue info to the nic.
        data.e1000_hw_ops.e1000_configure(&rx_ringbuf, &tx_ringbuf)?;

        *data.rx_ring.lock_irqdisable() = Some(rx_ringbuf);
        *data.tx_ring.lock_irqdisable() = Some(tx_ringbuf);

        let irq_prv_data = Box::try_new(IrqPrivateData{
            e1000_hw_ops: Arc::clone(&data.e1000_hw_ops),
            napi: Arc::clone(&data.napi),
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

        let mut tx_ring = data.tx_ring.lock_irqdisable();
        let mut tdt = data.e1000_hw_ops.e1000_read_tx_queue_tail();
        let tdh = data.e1000_hw_ops.e1000_read_tx_queue_head();
        let rdt = data.e1000_hw_ops.e1000_read_rx_queue_tail();
        let rdh = data.e1000_hw_ops.e1000_read_rx_queue_head();

        pr_info!("Rust for linux e1000 driver demo (net device start_xmit) tdt={}, tdh={}, rdt={}, rdh={}\n", tdt, tdh, rdt, rdh);

        /* On PCI/PCI-X HW, if packet size is less than ETH_ZLEN,
        * packets may get corrupted during padding by HW.
        * To WA this issue, pad all small packets manually.
        */
        skb.put_padto(bindings::ETH_ZLEN);
        
        // tell the kernel that we have pended some data to the hardware.
        dev.sent_queue(skb.len());

        let mut tx_ring = tx_ring.as_mut().unwrap();
        let tx_descs:&mut [TxDescEntry] = tx_ring.desc.as_desc_slice();
        let tx_desc = &mut tx_descs[tdt as usize];
        if tx_desc.sta & E1000_TXD_STAT_DD as u8 == 0 {
            pr_err!("xmit busy");
            return net::NetdevTx::Busy;
        }

        // alloc DMA map to skb
        let ms:dma::MapSingle<u8> = if let Ok(ms) = dma::MapSingle::try_new(&*data.dev, skb.head_data().as_ptr() as *mut u8, skb.len() as usize, bindings::dma_data_direction_DMA_TO_DEVICE) {
            ms
        } else {
            return net::NetdevTx::Busy;
        };

        tx_desc.buf_addr = ms.dma_handle as u64;
        tx_desc.length = skb.len() as u16;
        tx_desc.cmd = ((E1000_TXD_CMD_RS | E1000_TXD_CMD_EOP) >> 24) as u8;
        tx_desc.sta = 0;
        tx_ring.buf.borrow_mut()[tdt as usize].replace((ms, skb.into()));

        // TODO memory fence here. we are testing it on an x86, so maybe left it out is ok.

        tdt = (tdt + 1) % TX_RING_SIZE as u32;
        data.e1000_hw_ops.e1000_write_tx_queue_tail(tdt);

        
        net::NetdevTx::Ok
    }



    fn get_stats64(_netdev: &net::Device, _data: &NetDevicePrvData, stats: &mut net::RtnlLinkStats64) {
        pr_info!("Rust for linux e1000 driver demo (net device get_stats64)\n");
        // TODO not implemented.
        stats.set_rx_bytes(0);
        stats.set_rx_packets(0);
        stats.set_tx_bytes(0);
        stats.set_tx_packets(0);
    }
}

// since the ownership limitation, We can't use NetDevicePrvData as C code, so we need to define a new type here. 
struct IrqPrivateData {
    e1000_hw_ops: Arc<E1000Ops>,
    napi: Arc<net::Napi>,
}

struct E1000InterruptHandler {}

impl kernel::irq::Handler for E1000InterruptHandler {
    type Data = Box<IrqPrivateData>;

    fn handle_irq(data: &IrqPrivateData) -> kernel::irq::Return {
        pr_info!("Rust for linux e1000 driver demo (handle_irq)\n");

        let pending_irqs = data.e1000_hw_ops.e1000_read_interrupt_state();

        pr_info!("pending_irqs: {}\n", pending_irqs);

        if pending_irqs == 0 {
            return kernel::irq::Return::None
        }

        data.napi.schedule();

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

struct NapiHandler{}

impl net::NapiPoller for NapiHandler {
    type Data = Box<NetDevicePrvData>;

    fn poll(
        _napi: &net::Napi,
        _budget: i32,
        dev: &net::Device,
        data: &NetDevicePrvData,
    ) -> i32 {
        pr_info!("Rust for linux e1000 driver demo (napi poll)\n");

        let mut rdt = data.e1000_hw_ops.e1000_read_rx_queue_tail() as usize;
        rdt = (rdt + 1) % RX_RING_SIZE;



        let mut rx_ring_guard = data.rx_ring.lock();
        let rx_ring =  rx_ring_guard.as_mut().unwrap();

        
        let mut descs = rx_ring.desc.as_desc_slice();

        while descs[rdt].status & E1000_RXD_STAT_DD as u8 != 0 {
            let packet_len = descs[rdt].length as usize;
            let buf = &mut rx_ring.buf.borrow_mut();
            let skb = &buf[rdt].as_mut().unwrap().1;

            skb.put(packet_len as u32);
            let protocol = skb.eth_type_trans(dev);
            skb.protocol_set(protocol);

            data.napi.gro_receive(skb);

            let skb_new = dev.alloc_skb_ip_align(RXTX_SINGLE_RING_BLOCK_SIZE as u32).unwrap();
            let dma_map = dma::MapSingle::try_new(&*data.dev, skb_new.head_data().as_ptr() as *mut u8, RXTX_SINGLE_RING_BLOCK_SIZE, bindings::dma_data_direction_DMA_FROM_DEVICE).unwrap();
            descs[rdt].buf_addr = dma_map.dma_handle as u64;
            buf[rdt] = Some((dma_map, skb_new));

            descs[rdt].status = 0;
            data.e1000_hw_ops.e1000_write_rx_queue_tail(rdt as u32);
            rdt = (rdt + 1) % RX_RING_SIZE;
        }

        NetDevice::e1000_recycle_tx_queue(dev, data);
        data.napi.complete_done(1);
        1
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
        let mem_res = dev.iter_resource().next().ok_or(kernel::error::code::EIO)?;
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
        let napi = net::NapiAdapter::<NapiHandler>::add_weight(&netdev, 64)?;


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
                e1000_hw_ops: Arc::try_new(e1000_hw_ops)?,
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
