// the reg start physical addr of gic v2
// but I don't know the virtual addr now
// TODO
pub const GIC_V2_PPTR:usize = 0x8000000;
pub const GIC_V2_DISTRIBUTOR_PPTR: usize = GIC_V2_PPTR;
pub const GIC_V2_CONTROLLER_PPTR:usize = GIC_V2_DISTRIBUTOR_PPTR + 0x10000;
pub const GIC_V2_VCPUCTRL_PPTR:usize = GIC_V2_CONTROLLER_PPTR + 0x10000;