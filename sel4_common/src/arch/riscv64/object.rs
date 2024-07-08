use crate::sel4_config::{seL4_HugePageBits, seL4_LargePageBits, seL4_PageBits};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// Represents the type of an object.
pub enum ObjectType {
    UnytpedObject = 0,
    TCBObject = 1,
    EndpointObject = 2,
    NotificationObject = 3,
    CapTableObject = 4,
    // RISCV relevant object
    GigaPageObject = 5,
    NormalPageObject = 6,
    MegaPageObject = 7,
    PageTableObject = 8,
}

impl ObjectType {
    pub fn arch_get_object_size(&self) -> usize {
        match *self {
            ObjectType::GigaPageObject => seL4_HugePageBits,
            ObjectType::NormalPageObject => seL4_PageBits,
            ObjectType::MegaPageObject => seL4_LargePageBits,
            ObjectType::PageTableObject => seL4_PageBits,
            _ => panic!("unsupported cap type:{}", (*self) as usize),
        }
    }
}
