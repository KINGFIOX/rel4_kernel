use crate::arch::vm_rights_t;


pub fn vm_rights_from_word(w:usize)->vm_rights_t{
    unsafe{
        core::mem::transmute(w)
    }
}