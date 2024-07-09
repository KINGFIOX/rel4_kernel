//! zombie cap相关字段和方法
//! 当`tcb_cap`和`cnode_cap`删除的过程中会变为`zombie_cap`
use crate::cte::cte_t;
use sel4_common::sel4_config::wordRadix;
use sel4_common::MASK;

use super::{cap_t, CapTag};

/// Judge whether the zombie cap is from tcb cap.
pub const ZombieType_ZombieTCB: usize = 1usize << wordRadix;
pub const TCB_CNODE_RADIX: usize = 4;

/// zombie cap相关字段和方法
impl cap_t {
    #[inline]
    pub fn get_zombie_bit(&self) -> usize {
        let _type = self.get_zombie_type();
        if _type == ZombieType_ZombieTCB {
            return TCB_CNODE_RADIX;
        }
        return ZombieType_ZombieCNode(_type);
    }

    #[inline]
    pub fn get_zombie_ptr(&self) -> usize {
        let radix = self.get_zombie_bit();
        return self.get_zombie_id() & !MASK!(radix + 1);
    }

    #[inline]
    pub fn get_zombie_number(&self) -> usize {
        let radix = self.get_zombie_bit();
        return self.get_zombie_id() & MASK!(radix + 1);
    }

    #[inline]
    pub fn set_zombie_number(&mut self, n: usize) {
        let radix = self.get_zombie_bit();
        let ptr = self.get_zombie_id() & !MASK!(radix + 1);
        self.set_zombie_id(ptr | (n & MASK!(radix + 1)));
    }
}

#[inline]
pub fn Zombie_new(number: usize, _type: usize, ptr: usize) -> cap_t {
    let mask: usize;
    if _type == ZombieType_ZombieTCB {
        mask = MASK!(TCB_CNODE_RADIX + 1);
    } else {
        mask = MASK!(_type + 1);
    }
    return cap_t::new_zombie_cap((ptr & !mask) | (number & mask), _type);
}

pub fn ZombieType_ZombieCNode(n: usize) -> usize {
    return n & MASK!(wordRadix);
}

///判断是否为循环`zombie cap`,指向自身且类型为`CapZombieCap`（似乎只有`CNode Capability`指向自己才会出现这种情况）
/// 根据网上信息，当`cnode cap`为L2以上时，即`CNode`嵌套`CNode`的情况，就会产生`CyclicZombie`
#[inline]
#[no_mangle]
pub fn capCyclicZombie(cap: &cap_t, slot: *mut cte_t) -> bool {
    let ptr = cap.get_zombie_ptr() as *mut cte_t;
    (cap.get_cap_type() == CapTag::CapZombieCap) && (ptr == slot)
}
