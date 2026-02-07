pub(crate) const BRK_OPCODE: u32 = 0xD420_0000;
pub(crate) const BRK_MASK: u32 = 0xFFE0_001F;
pub(crate) const VM_PROT_COPY: libc::vm_prot_t = 0x10;
pub(crate) const MAX_INSTRUMENTS: usize = 256;

pub(crate) const LDR_X16_LITERAL_8: u32 = 0x5800_0050;
pub(crate) const BR_X16: u32 = 0xD61F_0200;
