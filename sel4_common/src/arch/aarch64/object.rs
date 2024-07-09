use crate::sel4_config::{
    seL4_PGDBits, seL4_PUDBits, seL4_PageDirBits, seL4_PageTableBits, ARMHugePageBits,
    ARMLargePageBits, ARMSmallPageBits, ARM_Huge_Page, ARM_Large_Page, ARM_Small_Page,
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

    /// Returns the frame type of the object.
    ///
    /// # Returns
    ///
    /// The frame type of the object.
    pub fn get_frame_type(&self) -> usize {
        match self {
            ObjectType::seL4_ARM_SmallPageObject => ARM_Small_Page,
            ObjectType::seL4_ARM_LargePageObject => ARM_Large_Page,
            ObjectType::seL4_ARM_HugePageObject => ARM_Huge_Page,
            _ => {
                panic!("Invalid frame type: {:?}", self);
            }
        }
    }
    /// Checks if the object type is an architecture-specific type.
    ///
    /// # Returns
    ///
    /// true if the object type is an architecture-specific type, false otherwise.
    pub fn is_arch_type(self) -> bool {
        match self {
            Self::seL4_ARM_HugePageObject
            | Self::seL4_ARM_PageUpperDirectoryObject
            | Self::seL4_ARM_PageGlobalDirectoryObject
            | Self::seL4_ARM_SmallPageObject
            | Self::seL4_ARM_LargePageObject
            | Self::seL4_ARM_PageTableObject
            | Self::seL4_ARM_PageDirectoryObject => true,
            _ => false,
        }
    }
}
