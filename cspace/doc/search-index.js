var searchIndex = JSON.parse('{\
"cspace":{"doc":"","t":"ANNNNNNNNNNNNENNNRFLLLLLLLLLLMMFRFRFFFFRFFFFFFFRFRFFFFFFFFFFFFRFFFFFFFFFFFFRFRFFRFFFFFFFFFRFRFFFFFFFFFRFFFFFDRFFRFFFFFFFRFFFFFFFFLLLLFMFFDLLLFDLFLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLFLLLLLFLFFFLLLFFFFFFFFFFDLLLLLLLLLLLLLLLLFLLLLLLLLLLLLLLLLLLLLLLLLLFMLLLLLLLLLLLLLLLMM","n":["interface","CapASIDControlCap","CapASIDPoolCap","CapCNodeCap","CapDomainCap","CapEndpointCap","CapFrameCap","CapIrqControlCap","CapIrqHandlerCap","CapNotificationCap","CapNullCap","CapPageTableCap","CapReplyCap","CapTag","CapThreadCap","CapUntypedCap","CapZombieCap","ZombieType_ZombieTCB","Zombie_new","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","cap","cap","capRemovable","cap_asid_control_cap","cap_asid_control_cap_new","cap_asid_pool_cap","cap_asid_pool_cap_get_capASIDBase","cap_asid_pool_cap_get_capASIDPool","cap_asid_pool_cap_new","cap_capType_equals","cap_cnode_cap","cap_cnode_cap_get_capCNodeGuard","cap_cnode_cap_get_capCNodeGuardSize","cap_cnode_cap_get_capCNodePtr","cap_cnode_cap_get_capCNodeRadix","cap_cnode_cap_new","cap_cnode_cap_set_capCNodeGuard","cap_cnode_cap_set_capCNodeGuardSize","cap_domain_cap","cap_domain_cap_new","cap_endpoint_cap","cap_endpoint_cap_get_capCanGrant","cap_endpoint_cap_get_capCanGrantReply","cap_endpoint_cap_get_capCanReceive","cap_endpoint_cap_get_capCanSend","cap_endpoint_cap_get_capEPBadge","cap_endpoint_cap_get_capEPPtr","cap_endpoint_cap_new","cap_endpoint_cap_set_capCanGrant","cap_endpoint_cap_set_capCanGrantReply","cap_endpoint_cap_set_capCanReceive","cap_endpoint_cap_set_capCanSend","cap_endpoint_cap_set_capEPBadge","cap_frame_cap","cap_frame_cap_get_capFBasePtr","cap_frame_cap_get_capFIsDevice","cap_frame_cap_get_capFMappedASID","cap_frame_cap_get_capFMappedAddress","cap_frame_cap_get_capFSize","cap_frame_cap_get_capFVMRights","cap_frame_cap_new","cap_frame_cap_set_capFMappedASID","cap_frame_cap_set_capFMappedAddress","cap_frame_cap_set_capFVMRights","cap_get_capPtr","cap_get_capType","cap_irq_control_cap","cap_irq_control_cap_new","cap_irq_handler_cap","cap_irq_handler_cap_get_capIRQ","cap_irq_handler_cap_new","cap_notification_cap","cap_notification_cap_get_capNtfnBadge","cap_notification_cap_get_capNtfnCanReceive","cap_notification_cap_get_capNtfnCanSend","cap_notification_cap_get_capNtfnPtr","cap_notification_cap_new","cap_notification_cap_set_capNtfnBadge","cap_notification_cap_set_capNtfnCanReceive","cap_notification_cap_set_capNtfnCanSend","cap_notification_cap_set_capNtfnPtr","cap_null_cap","cap_null_cap_new","cap_page_table_cap","cap_page_table_cap_get_capPTBasePtr","cap_page_table_cap_get_capPTIsMapped","cap_page_table_cap_get_capPTMappedASID","cap_page_table_cap_get_capPTMappedAddress","cap_page_table_cap_new","cap_page_table_cap_ptr_set_capPTIsMapped","cap_page_table_cap_set_capPTIsMapped","cap_page_table_cap_set_capPTMappedASID","cap_page_table_cap_set_capPTMappedAddress","cap_reply_cap","cap_reply_cap_get_capReplyCanGrant","cap_reply_cap_get_capReplyMaster","cap_reply_cap_get_capTCBPtr","cap_reply_cap_new","cap_reply_cap_set_capReplyCanGrant","cap_t","cap_thread_cap","cap_thread_cap_get_capTCBPtr","cap_thread_cap_new","cap_untyped_cap","cap_untyped_cap_get_capBlockSize","cap_untyped_cap_get_capFreeIndex","cap_untyped_cap_get_capIsDevice","cap_untyped_cap_get_capPtr","cap_untyped_cap_new","cap_untyped_cap_ptr_set_capFreeIndex","cap_untyped_cap_set_capFreeIndex","cap_zombie_cap","cap_zombie_cap_get_capZombieBits","cap_zombie_cap_get_capZombieID","cap_zombie_cap_get_capZombieNumber","cap_zombie_cap_get_capZombiePtr","cap_zombie_cap_get_capZombieType","cap_zombie_cap_new","cap_zombie_cap_set_capZombieID","cap_zombie_cap_set_capZombieNumber","clone","clone","clone","clone","cteInsert","cteMDBNode","cteMove","cteSwap","cte_t","default","default","default","deriveCap","deriveCap_ret","derive_cap","ensureNoChildren","ensure_no_children","eq","eq","eq","fmt","fmt","fmt","from","from","from","from","from","get_asid_base","get_asid_pool","get_cap_is_physical","get_cap_ptr","get_cap_size_bits","get_cap_type","get_cnode_guard","get_cnode_guard_size","get_cnode_ptr","get_cnode_radix","get_ep_badge","get_ep_can_grant","get_ep_can_grant_reply","get_ep_can_receive","get_ep_can_send","get_ep_ptr","get_first_badged","get_frame_base_ptr","get_frame_is_device","get_frame_mapped_address","get_frame_mapped_asid","get_frame_size","get_frame_vm_rights","get_irq_handler","get_next","get_nf_badge","get_nf_can_receive","get_nf_can_send","get_nf_ptr","get_prev","get_pt_base_ptr","get_pt_is_mapped","get_pt_mapped_address","get_pt_mapped_asid","get_reply_can_grant","get_reply_master","get_reply_tcb_ptr","get_revocable","get_tcb_ptr","get_untyped_block_size","get_untyped_free_index","get_untyped_is_device","get_untyped_ptr","get_zombie_bit","get_zombie_id","get_zombie_number","get_zombie_ptr","get_zombie_type","insertNewCap","into","into","into","into","into","isArchCap","isArchCap","isCapRevocable","isFinalCapability","isMDBParentOf","is_final_cap","is_long_running_delete","is_mdb_parent_of","mdb_node_get_mdbFirstBadged","mdb_node_get_mdbNext","mdb_node_get_mdbPrev","mdb_node_get_mdbRevocable","mdb_node_new","mdb_node_ptr_set_mdbNext","mdb_node_ptr_set_mdbPrev","mdb_node_set_mdbFirstBadged","mdb_node_set_mdbPrev","mdb_node_set_mdbRevocable","mdb_node_t","new","new_asid_control_cap","new_asid_pool_cap","new_cnode_cap","new_domain_cap","new_endpoint_cap","new_frame_cap","new_irq_control_cap","new_irq_handler_cap","new_notification_cap","new_null_cap","new_page_table_cap","new_reply_cap","new_thread_cap","new_untyped_cap","new_zombie_cap","sameObjectAs","set_cnode_guard","set_cnode_guard_size","set_ep_badge","set_ep_can_grant","set_ep_can_grant_reply","set_ep_can_receive","set_ep_can_send","set_first_badged","set_frame_mapped_address","set_frame_mapped_asid","set_frame_vm_rights","set_next","set_nf_badge","set_nf_can_receive","set_nf_can_send","set_nf_ptr","set_prev","set_pt_is_mapped","set_pt_mapped_address","set_pt_mapped_asid","set_reply_can_grant","set_revocable","set_untyped_free_index","set_zombie_id","set_zombie_number","slotCapLongRunningDelete","status","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","words","words"],"q":[[0,"cspace"],[1,"cspace::interface"]],"d":["","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"i":[0,8,8,8,8,8,8,8,8,8,8,8,8,0,8,8,8,0,0,8,2,5,6,3,8,2,5,6,3,6,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,5,6,3,0,3,0,0,0,2,5,3,0,0,3,0,3,8,2,5,8,2,5,8,2,5,6,3,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,5,2,2,2,2,2,2,2,5,2,2,2,2,5,2,2,2,2,2,2,2,5,2,2,2,2,2,2,2,2,2,2,0,8,2,5,6,3,0,2,0,0,0,3,3,3,0,0,0,0,0,0,0,0,0,0,0,5,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,0,2,2,2,2,2,2,2,5,2,2,2,5,2,2,2,2,5,2,2,2,2,5,2,2,2,0,6,8,2,5,6,3,8,2,5,6,3,8,2,5,6,3,2,5],"f":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[1,1,1],2],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],0,0,[[2,3],4],0,[[],2],0,[2,1],[2,1],[[1,1],2],[[2,1],4],0,[2,1],[2,1],[2,1],[2,1],[[1,1,1,1],2],[[2,1]],[[2,1]],0,[[],2],0,[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[[1,1,1,1,1,1],2],[[2,1]],[[2,1]],[[2,1]],[[2,1]],[[2,1]],0,[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[[1,1,1,1,1,1],2],[[2,1]],[[2,1]],[[2,1]],[2,1],[2,1],0,[[],2],0,[2,1],[1,2],0,[2,1],[2,1],[2,1],[2,1],[[1,1,1,1],2],[[2,1]],[[2,1]],[[2,1]],[[2,1]],0,[[],2],0,[2,1],[2,1],[2,1],[2,1],[[1,1,1,1],2],[[2,1]],[[2,1]],[[2,1]],[[2,1]],0,[2,1],[2,1],[2,1],[[1,1,1],2],[[2,1]],0,0,[2,1],[1,2],0,[2,1],[2,1],[2,1],[2,1],[[1,1,1,1],2],[[2,1]],[[2,1]],0,[2,1],[2,1],[2,1],[2,1],[2,1],[[1,1],2],[[2,1]],[[2,1]],[2,2],[5,5],[6,6],[3,3],[[2,3,3]],0,[[2,3,3]],[[2,3,2,3]],0,[[],2],[[],5],[[],3],[[3,2],6],0,[[3,2],6],[3,7],[3,7],[[8,8],4],[[2,2],4],[[5,5],4],[[8,9],10],[[2,9],10],[[5,9],10],[[]],[[]],[[]],[[]],[[]],[2,1],[2,1],[2,4],[2,1],[2,1],[2,8],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[5,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[5,1],[2,1],[2,1],[2,1],[2,1],[5,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[5,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[2,1],[[3,3,2]],[[]],[[]],[[]],[[]],[[]],[2,4],[2,4],[[2,2],4],[3,4],[[3,3],4],[3,4],[3,4],[[3,3],4],[5,1],[5,1],[5,1],[5,1],[[1,1,1,1],5],[[5,1]],[[5,1]],[[5,1]],[[5,1]],[[5,1]],0,[[1,1,1,1],5],[[],2],[[1,1],2],[[1,1,1,1],2],[[],2],[[1,1,1,1,1,1],2],[[1,1,1,1,1,1],2],[[],2],[1,2],[[1,1,1,1],2],[[],2],[[1,1,1,1],2],[[1,1,1],2],[1,2],[[1,1,1,1],2],[[1,1],2],[[2,2],4],[[2,1]],[[2,1]],[[2,1]],[[2,1]],[[2,1]],[[2,1]],[[2,1]],[[5,1]],[[2,1]],[[2,1]],[[2,1]],[[5,1]],[[2,1]],[[2,1]],[[2,1]],[[2,1]],[[5,1]],[[2,1]],[[2,1]],[[2,1]],[[2,1]],[[5,1]],[[2,1]],[[2,1]],[[2,1]],[3,4],0,[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],12],[[],12],[[],12],[[],12],[[],12],0,0],"c":[],"p":[[15,"usize"],[3,"cap_t"],[3,"cte_t"],[15,"bool"],[3,"mdb_node_t"],[3,"deriveCap_ret"],[4,"exception_t"],[4,"CapTag"],[3,"Formatter"],[6,"Result"],[4,"Result"],[3,"TypeId"]]}\
}');
if (typeof window !== 'undefined' && window.initSearch) {window.initSearch(searchIndex)};
if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};
