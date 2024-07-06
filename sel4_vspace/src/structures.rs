use core::{
    ffi::CStr,
    fmt::{Debug, Display},
};

use sel4_common::{
    fault::lookup_fault_t,
    sel4_config::{asidLowBits, PPTR_BASE},
    structures::exception_t,
    utils::convert_to_option_mut_type_ref,
    BIT,
};

use crate::pte_t;

/// 在`PSpace`段的虚拟地址空间中的指针
///
/// Virtual pointer used in PSpace
pub type pptr_t = usize;
/// 用戶地址地址空间中虚拟的指针
///
/// Virtual pointer in user space
pub type vptr_t = usize;

/// 进程对应的asid所属的类型
pub type asid_t = usize;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct findVSpaceForASID_ret {
    pub status: exception_t,
    pub vspace_root: Option<*mut pte_t>,
    pub lookup_fault: Option<lookup_fault_t>,
}

/// 用于存放`asid`对应的根页表基址，是一个`usize`的数组，其中`asid`按低`asidLowBits`位进行索引
#[derive(Copy, Clone)]
pub struct asid_pool_t {
    pub array: [*mut pte_t; BIT!(asidLowBits)],
}

/// `asid pool`相关操作
impl asid_pool_t {
    #[inline]
    pub fn get_ptr(&self) -> pptr_t {
        self as *const Self as pptr_t
    }

    #[inline]
    pub fn get_vspace_by_index(&mut self, index: usize) -> Option<&'static mut pte_t> {
        convert_to_option_mut_type_ref::<pte_t>(self.array[index] as usize)
    }

    #[inline]
    pub fn set_vspace_by_index(&mut self, index: usize, vspace_ptr: pptr_t) {
        // assert!(index < BIT!(asidLowBits));
        self.array[index] = vspace_ptr as *mut pte_t;
    }
}

/// 进行系统调用时，应用程序向内核传递信息的消息格式
///
/// vm_attributes_t is a message type. When program pass message to kernel , it uses vm_attributes_t.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct vm_attributes_t(pub(crate) usize);

impl vm_attributes_t {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn from_word(w: usize) -> Self {
        Self::new(w)
    }

    pub fn get_execute_never(&self) -> usize {
        self.0 & 0x1usize
    }

    pub fn set_execute_never(&mut self, v64: usize) {
        self.0 &= !0x1usize;
        self.0 |= (v64 << 0) & 0x1usize;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct paddr_t(pub(crate) usize);
impl From<usize> for paddr_t {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl paddr_t {
    #[inline]
    pub fn addr(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn get_ptr<T>(&self) -> *const T {
        (self.0 | PPTR_BASE) as *const T
    }

    #[inline]
    pub const fn get_mut_ptr<T>(&self) -> *mut T {
        (self.0 | PPTR_BASE) as *mut T
    }

    #[inline]
    pub fn slice_with_len<T>(&self, len: usize) -> &'static [T] {
        unsafe { core::slice::from_raw_parts(self.get_ptr(), len) }
    }

    #[inline]
    pub fn slice_mut_with_len<T>(&self, len: usize) -> &'static mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.get_mut_ptr(), len) }
    }

    #[inline]
    pub fn get_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.get_ptr::<i8>()) }
    }
}

impl Debug for paddr_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for paddr_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}
