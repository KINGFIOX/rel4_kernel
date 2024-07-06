use sel4_common::plus_define_bitfield;

/// Cap 在内核态中的种类枚举
#[derive(Eq, PartialEq, Debug)]
pub enum CapTag {
    CapNullCap = 0,
    CapUntypedCap = 2,
    CapEndpointCap = 4,
    CapNotificationCap = 6,
    CapReplyCap = 8,
    CapCNodeCap = 10,
    CapThreadCap = 12,
    CapIrqControlCap = 14,
    CapIrqHandlerCap = 16,
    CapZombieCap = 18,
    CapDomainCap = 20,
    CapFrameCap = 1,
    CapPageTableCap = 3,
    CapPageDirectoryCap = 5,
    CapPageUpperDirectoryCap = 7,
    CapPageGlobalDirectoryCap = 9,
    CapASIDControlCap = 11,
    CapASIDPoolCap = 13,
}

plus_define_bitfield! {
    cap_t, 2, 0, 59, 5 => {
        new_frame_cap, CapTag::CapFrameCap as usize => {
            capFIsDevice, get_frame_is_device,set_frame_is_device, 0, 6, 1, 0, false,
            capFVMRights,get_frame_vm_rights, set_frame_vm_rights, 0, 7, 2, 0, false,
            capFMappedAddress, get_frame_mapped_address, set_frame_mapped_address, 0, 9, 48, 0, true,
            capFSize, get_frame_size, set_frame_size, 0, 57, 2, 0, false,
            capFMappedASID, get_frame_mapped_asid, set_frame_mapped_asid, 1, 48, 16, 0, false,
            capFBasePtr, get_frame_base_ptr, set_frame_base_ptr, 1, 0, 48, 0, true

        },
        new_page_table_cap, CapTag::CapPageTableCap as usize => {
            capPTMappedASID, get_pt_mapped_asid, set_pt_mapped_asid, 1, 48, 16, 0, false,
            capPTBasePtr, get_pt_base_ptr, set_pt_base_ptr, 1, 0, 48, 0, true,
            capPTIsMapped, get_pt_is_mapped, set_pt_is_mapped, 0, 48, 1, 0, false,
            capPTMappedAddress, get_pt_mapped_address, set_pt_mapped_address, 0, 20, 28, 0, false
        },
        new_page_directory_cap, CapTag::CapPageDirectoryCap as usize => {
            capPDMappedASID, get_pd_mapped_asid, set_pd_mapped_asid, 1, 48, 16, 0, false,
            capPDBasePtr, get_pd_base_ptr, set_pd_base_ptr, 1, 0, 48, 0, true,
            capPDIsMapped, get_pd_is_mapped, set_pd_is_mapped, 0, 48, 1, 0, false,
            capPDMappedAddress, get_pd_mapped_address, set_pd_mapped_address, 0, 29, 19, 0, false
        },
        new_page_upper_directory_cap, CapTag::CapPageUpperDirectoryCap as usize => {
            capPUDMappedASID, get_pud_mapped_asid, set_pud_mapped_asid, 1, 48, 16, 0, false,
            capPUDBasePtr, get_pud_base_ptr, set_pud_base_ptr, 1, 0, 48, 0, true,
            capPUDIsMapped, get_pud_is_mapped, set_pud_is_mapped, 0, 58, 1, 0, false,
            capPUDMappedAddress, get_pud_mapped_address, set_pud_mapped_address, 0, 48, 10, 0, false
        },
        new_page_global_directory_cap, CapTag::CapPageGlobalDirectoryCap as usize => {
            capPGDMappedASID, get_pgd_mapped_asid, set_pgd_mapped_asid, 1, 48, 16, 0, false,
            capPGDBasePtr, get_pgd_base_ptr, set_pgd_base_ptr, 1, 0, 48, 0, true,
            capPGDIsMapped, get_pgd_is_mapped, set_pgd_is_mapped, 0, 58, 1, 0, false
        },
        new_asid_control_cap, CapTag::CapASIDControlCap as usize => {},
        new_asid_pool_cap, CapTag::CapASIDPoolCap as usize => {
            capASIDBase, get_asid_base, set_asid_base, 0, 43, 16, 0, false,
            capASIDPool, get_asid_pool, set_asid_pool, 0, 0, 37, 2, true
        }
    }
}
