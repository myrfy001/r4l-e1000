use kernel::net::SkBuff;
use kernel::prelude::*;
use kernel::dma;
use core::cell::RefCell;
use crate::hw_defs::{RxDescEntry,TxDescEntry};


pub(crate) struct RingBuf<T> {
    pub(crate) desc: dma::Allocation::<T>,
    pub(crate) buf: RefCell<Vec<Option<(dma::MapSingle::<u8>,ARef<SkBuff>)>>>,
    pub(crate) next_to_clean: usize,
    len: usize,
    block_size: usize,
}

impl<T> RingBuf<T> {
    pub(crate) fn as_desc_slice(&self) -> &mut [T] {
        unsafe{core::slice::from_raw_parts_mut(self.desc.cpu_addr, self.len)}
    }


    pub(crate) fn new(desc: dma::Allocation::<T>, len: usize, block_size: usize) -> Self {
        let buf = RefCell::new(Vec::new());
        
        {
            let mut buf_ref = buf.borrow_mut();
            for _ in 0..len{
                buf_ref.try_push(None).unwrap();
            }
        }
        Self {desc, buf, len, block_size, next_to_clean:0}
    }
}

pub(crate) type RxRingBuf = RingBuf<RxDescEntry>;
pub(crate) type TxRingBuf = RingBuf<TxDescEntry>;