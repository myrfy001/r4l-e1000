use kernel::net::SkBuff;
use kernel::prelude::*;
use kernel::dma;
use core::cell::RefCell;
use crate::hw_defs::{RxDescEntry,TxDescEntry};

/// A pair made up of a SkBuff and it's dma mapping
pub(crate) type SkbDma = (dma::MapSingle::<u8>, ARef<SkBuff>);

/// A slice view into `dma::Allocation`.
pub(crate) struct DmaAllocSlice<T> {
    desc: dma::Allocation::<T>,
    count: usize,
}


impl<T> DmaAllocSlice<T> {
    pub(crate) fn as_desc_slice(&mut self) -> &mut [T] {
        unsafe{core::slice::from_raw_parts_mut(self.desc.cpu_addr, self.count)}
    }

    pub(crate) fn get_dma_addr(&self) -> usize {
        self.desc.dma_handle as usize
    }

    pub(crate) fn get_cpu_addr(&self) -> usize {
        self.desc.cpu_addr as usize
    }
}

pub(crate) struct RingBuf<T> {
    pub(crate) desc: DmaAllocSlice<T>,
    pub(crate) buf: RefCell<Vec<Option<SkbDma>>>,
    pub(crate) next_to_clean: usize,
}

impl<T> RingBuf<T> {
    pub(crate) fn new(desc: dma::Allocation::<T>, len: usize) -> Self {
        let buf = RefCell::new(Vec::new());
        
        {
            let mut buf_ref = buf.borrow_mut();
            for _ in 0..len{
                buf_ref.try_push(None).unwrap();
            }
        }

        let desc = DmaAllocSlice{
            desc,
            count: len,
        };
        Self {desc, buf, next_to_clean:0}
    }
}

pub(crate) type RxRingBuf = RingBuf<RxDescEntry>;
pub(crate) type TxRingBuf = RingBuf<TxDescEntry>;