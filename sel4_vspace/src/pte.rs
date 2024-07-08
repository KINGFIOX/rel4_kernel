//! 页表项的相关操作，`map``unmap`等

/// 页表项（`page table entry`）
#[repr(C)]
#[derive(Copy, Clone)]
pub struct pte_t(pub usize);

impl pte_t {
    #[inline]
    pub fn get_ptr(&self) -> usize {
        self as *const Self as usize
    }
}
