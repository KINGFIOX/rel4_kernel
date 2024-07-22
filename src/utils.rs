use crate::BIT;

#[cfg(target_arch = "riscv64")]
#[inline]
pub fn clear_memory(ptr: *mut u8, bits: usize) {
    unsafe {
        core::slice::from_raw_parts_mut(ptr, BIT!(bits)).fill(0);
    }
}

// /* Cleaning memory before user-level access */
// static inline void clearMemory(word_t *ptr, word_t bits)
// {
//     memzero(ptr, (1ul << (bits)));
//     cleanCacheRange_RAM((word_t)ptr, (word_t)ptr + (1ul << (bits)) - 1,
//                         addrFromPPtr(ptr));
// }

#[cfg(target_arch = "aarch64")]
#[inline]
pub fn clear_memory(ptr: *mut u8, bits: usize) {
    use sel4_vspace::{clean_cache_range_ram, pptr_to_paddr};

    unsafe {
        core::slice::from_raw_parts_mut(ptr, BIT!(bits)).fill(0);
        clean_cache_range_ram(
            ptr as usize,
            ptr.add(BIT!(bits) - 1) as usize,
            pptr_to_paddr(ptr as usize),
        );
    }
}

// static inline void clearMemory_PT(word_t *ptr, word_t bits)
// {
//     memzero(ptr, (1ul << (bits)));
//     cleanCacheRange_PoU((word_t)ptr, (word_t)ptr + (1ul << (bits)) - 1,
//                         addrFromPPtr(ptr));
// }

#[cfg(target_arch = "aarch64")]
#[inline]
pub fn clear_memory_pt(ptr: *mut u8, bits: usize) {
    use sel4_vspace::{clean_cache_range_pou, pptr_to_paddr};

    unsafe {
        core::slice::from_raw_parts_mut(ptr, BIT!(bits)).fill(0);
        clean_cache_range_pou(
            ptr as usize,
            ptr.add(BIT!(bits) - 1) as usize,
            pptr_to_paddr(ptr as usize),
        );
    }
}
