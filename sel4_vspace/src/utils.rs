use sel4_common::{utils::pageBitsForSize, MASK};

#[inline]
#[no_mangle]
pub fn checkVPAlignment(sz: usize, w: usize) -> bool {
    w & MASK!(pageBitsForSize(sz)) == 0
}
