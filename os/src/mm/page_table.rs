//! Implementation of [`PageTableEntry`] and [`PageTable`].

use crate::task::{append_to_memset, pay_back, push};

use super::{
    frame_alloc, FrameTracker, MapPermission, PhysPageNum, StepByOne, VirtAddr, VirtPageNum
};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    /// bits of page table entry
    pub bits: usize,
}

impl PageTableEntry {
    /// Create a new page table entry
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    /// Create an empty page table entry
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    /// Get the physical page number from the page table entry
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    /// Get the flags from the page table entry
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    /// The page pointered by page table entry is valid?
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is readable?
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is writable?
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is executable?
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// page table structure
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    /// Create a new page table
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    /// Find PageTableEntry by VirtPageNum, create a frame for a 4KB page table if not exist
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    /// Find PageTableEntry by VirtPageNum
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                // println!("here bug");
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    /// set the map between virtual page number and physical page number
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) -> bool {
        let pte = self.find_pte_create(vpn).unwrap();
        if pte.is_valid() {
            return false;
        }
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
        //println!("{}");
        true
    }
    /// remove the map between virtual page number and physical page number
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) -> bool {
        let pte = self.find_pte(vpn).unwrap();
        // assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        if !pte.is_valid() {
            return false;
        }
        *pte = PageTableEntry::empty();
        true
    }
    /// get the page table entry from the virtual page number
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    /// get the token from the page table
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

/// Translate&Copy a ptr[u8] array with LENGTH len to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

/// get ppn
pub fn tran_vir_to_phy(usr_token: usize, vaddr: VirtAddr) -> PhysPageNum {
    let page_table = PageTable::from_token(usr_token);
    let vpn = vaddr.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    // let kernel_token = KERNEL_SPACE.exclusive_access().token();
    // PageTable::from_token(kernel_token).map(vpn, ppn, PTEFlags::R | PTEFlags::W | PTEFlags::X);
    ppn

    // let page = ppn.get_mut();
}

/// do a map
pub fn successful_map(
    // token: usize,
    start: usize,
    //mut vpn_vec: Vec<VirtPageNum>,
    len: usize,
    port: usize,
) -> bool {
    // let memory_set = get_memory_set();
    let end = start + len;
    let mut map_perm = MapPermission::U;
    // bits += 1; // v
    if port & 0x1 == 1 {
        map_perm |= MapPermission::R; // r
    }
    if (port >> 1) & 0x1 == 1 {
        map_perm |= MapPermission::W; // w
    }
    if (port >> 2) & 0x1 == 1 {
        map_perm |= MapPermission::X; // x
    }
    // let pte = self.find_pte_create(vpn).unwrap();
    // assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
    // let memory_area = MapArea::new(start.into(), (end).into(), MapType::Framed, map_perm);
    if !push(start, end, map_perm) {
        return false;
    }

    if append_to_memset(start, end) {
        true
    } else {
        false
    }

    /*memory_set.push(
        memory_area,
        None,
    );
    if memory_set.append_to(start.into(), end.into()) {
        return true;
    } else {
        false
    }*/

    // MemorySet

    /*let mut page_table = PageTable::from_token(token);
    let end = start + len;
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        println!("pagenum:{}", vpn.0);
        if page_table.translate(vpn).is_none() {
            // 说明该段虚拟地址空间未被映射， 符合要求
            // 申请物理内存 按页分配并映射
            let frame = frame_alloc().unwrap();
            let ppn = frame.ppn;
            // page_table.frames.push(frame);
            let mut bits: u8 = 0;
            // bits += 1; // v
            if port & 0x1 == 1 {
                bits += 2; // r
            }
            if (port >> 1) & 0x1 == 1 {
                bits += 4; // w
            }
            if (port >> 2) & 0x1 == 1 {
                bits += 8; // x
            }
            bits += 16; // 置U位为1

            let pte_flags = PTEFlags::from_bits(bits).unwrap();
            page_table.map(vpn, ppn, pte_flags);

            // println!("{}", pte_flags.bits);
        } else {
            //println!("地址空间已存在映射");
            println!("bug here");
            return false;
        }
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        start = end_va.into();
    }
    true*/
}

/// unmap
pub fn successful_unmap(token: usize, mut start: usize, len: usize) -> bool {
    // let page_count = len / 4096;
    let begin = start;
    let mut page_table = PageTable::from_token(token);
    let end = start + len;

    while start < end {
        // 先进先出
        //let vpn = vpn_vec.pop().unwrap();
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        if page_table.translate(vpn).is_none() {
            return false;
        } else {
            if !page_table.unmap(vpn) {
                return false;
            }
        }
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        start = end_va.into();
    }
    pay_back(begin, end);
    true
}
