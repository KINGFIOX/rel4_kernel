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
        new_null_cap, CapTag::CapNullCap as usize => {},
        new_untyped_cap, CapTag::CapUntypedCap as usize => {
            capFreeIndex, get_untyped_free_index, set_untyped_free_index, 1, 25, 39, 0, false,
            capIsDevice, get_untyped_is_device, set_untyped_is_device, 1, 6, 1, 0, false,
            capBlockSize, get_untyped_block_size, set_untyped_block_size, 1, 0, 6, 0, false,
            capPtr, get_untyped_ptr, set_untyped_ptr, 0, 0, 39, 0, true
        },
        new_endpoint_cap, CapTag::CapEndpointCap as usize => {
            capEPBadge, get_ep_badge, set_ep_badge, 1, 0, 64, 0, false,
            capCanGrantReply, get_ep_can_grant_reply, set_ep_can_grant_reply, 0, 58, 1, 0, false,
            capCanGrant, get_ep_can_grant, set_ep_can_grant, 0, 57, 1, 0, false,
            capCanSend, get_ep_can_send, set_ep_can_send, 0, 55, 1, 0, false,
            capCanReceive, get_ep_can_receive, set_ep_can_receive, 0, 56, 1, 0, false,
            capEPPtr, get_ep_ptr, set_ep_ptr, 0, 0, 39, 0, true
        },
        new_notification_cap, CapTag::CapNotificationCap as usize => {
            capNtfnBadge, get_nf_badge, set_nf_badge, 1, 0, 64, 0, false,
            capNtfnCanReceive, get_nf_can_receive, set_nf_can_receive, 0, 58, 1, 0, false,
            capNtfnCanSend, get_nf_can_send, set_nf_can_send, 0, 57, 1, 0, false,
            capNtfnPtr, get_nf_ptr, set_nf_ptr, 0, 0, 39, 0, true
        },
        new_reply_cap, CapTag::CapReplyCap as usize => {
            capReplyCanGrant, get_reply_can_grant, set_reply_can_grant, 0, 1, 1, 0, false,
            capReplyMaster, get_reply_master, set_reply_master, 0, 0, 1, 0, false,
            capTCBPtr, get_reply_tcb_ptr, set_reply_tcb_ptr, 1, 0, 64, 0, false
        },
        new_cnode_cap, CapTag::CapCNodeCap as usize => {
            capCNodeRadix, get_cnode_radix, set_cnode_radix, 0, 47, 6, 0, false,
            capCNodeGuardSize, get_cnode_guard_size, set_cnode_guard_size, 0, 53, 6, 0, false,
            capCNodeGuard, get_cnode_guard, set_cnode_guard, 1, 0, 64, 0, false,
            capCNodePtr, get_cnode_ptr, set_cnode_ptr, 0, 0, 38, 1, true
        },
        new_thread_cap, CapTag::CapThreadCap as usize => {
            capTCBPtr, get_tcb_ptr, set_tcb_ptr, 0, 0, 39, 0, true
        },
        new_irq_control_cap, CapTag::CapIrqControlCap as usize => {},
        new_irq_handler_cap, CapTag::CapIrqHandlerCap as usize => {
            capIRQ, get_irq_handler, set_irq_handler, 1, 0, 12, 0, false
        },
        new_zombie_cap, CapTag::CapZombieCap as usize => {
            capZombieID, get_zombie_id, set_zombie_id, 1, 0, 64, 0, false,
            capZombieType, get_zombie_type, set_zombie_type, 0, 0, 7, 0, false
        },
        new_domain_cap, CapTag::CapDomainCap as usize => {},
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
