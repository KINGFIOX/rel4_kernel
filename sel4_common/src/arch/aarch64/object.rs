use crate::sel4_config::{
    seL4_PGDBits, seL4_PUDBits, seL4_PageDirBits, seL4_PageTableBits, ARMHugePageBits,
    ARMLargePageBits, ARMSmallPageBits,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// Represents the type of an object.
pub enum ObjectType {
    UnytpedObject = 0,
    TCBObject = 1,
    EndpointObject = 2,
    NotificationObject = 3,
    CapTableObject = 4,
    seL4_ARM_HugePageObject = 5,
    seL4_ARM_PageUpperDirectoryObject = 6,
    seL4_ARM_PageGlobalDirectoryObject = 7,
    seL4_ARM_SmallPageObject = 8,
    seL4_ARM_LargePageObject = 9,
    seL4_ARM_PageTableObject = 10,
    seL4_ARM_PageDirectoryObject = 11,
}

impl ObjectType {
    pub fn arch_get_object_size(&self) -> usize {
        match self {
            Self::seL4_ARM_SmallPageObject => ARMSmallPageBits,
            Self::seL4_ARM_LargePageObject => ARMLargePageBits,
            Self::seL4_ARM_HugePageObject => ARMHugePageBits,
            Self::seL4_ARM_PageTableObject => seL4_PageTableBits,
            Self::seL4_ARM_PageUpperDirectoryObject => seL4_PUDBits,
            Self::seL4_ARM_PageDirectoryObject => seL4_PageDirBits,
            Self::seL4_ARM_PageGlobalDirectoryObject => seL4_PGDBits,
            _ => panic!("unsupported object type:{}", *self as usize),
        }
    }
}
