use crate::constants::MAX_INSTRUMENTS;
use crate::context::InstrumentCallback;
use crate::error::SigHookError;
use crate::trampoline;

#[derive(Copy, Clone)]
pub(crate) struct InstrumentSlot {
    pub used: bool,
    pub address: u64,
    pub original_opcode: u32,
    pub callback: Option<InstrumentCallback>,
    pub execute_original: bool,
    pub trampoline_pc: u64,
}

impl InstrumentSlot {
    pub const EMPTY: Self = Self {
        used: false,
        address: 0,
        original_opcode: 0,
        callback: None,
        execute_original: false,
        trampoline_pc: 0,
    };
}

pub(crate) static mut HANDLERS_INSTALLED: bool = false;
pub(crate) static mut SLOTS: [InstrumentSlot; MAX_INSTRUMENTS] =
    [InstrumentSlot::EMPTY; MAX_INSTRUMENTS];

pub(crate) unsafe fn find_slot_index(address: u64) -> Option<usize> {
    let mut index = 0;
    while index < MAX_INSTRUMENTS {
        let slot = unsafe { SLOTS[index] };
        if slot.used && slot.address == address {
            return Some(index);
        }
        index += 1;
    }
    None
}

pub(crate) unsafe fn slot_by_address(address: u64) -> Option<InstrumentSlot> {
    let index = unsafe { find_slot_index(address) }?;
    Some(unsafe { SLOTS[index] })
}

pub(crate) unsafe fn register_slot(
    address: u64,
    original_opcode: u32,
    callback: InstrumentCallback,
    execute_original: bool,
) -> Result<(), SigHookError> {
    if let Some(index) = unsafe { find_slot_index(address) } {
        let mut slot = unsafe { SLOTS[index] };

        slot.callback = Some(callback);
        slot.execute_original = execute_original;

        if slot.original_opcode == 0 {
            slot.original_opcode = original_opcode;
        }

        if execute_original && slot.trampoline_pc == 0 {
            slot.trampoline_pc = trampoline::create_original_trampoline(
                address.wrapping_add(4),
                slot.original_opcode,
            )?;
        }

        unsafe {
            SLOTS[index] = slot;
        }

        return Ok(());
    }

    let mut index = 0;
    while index < MAX_INSTRUMENTS {
        if !(unsafe { SLOTS[index].used }) {
            let trampoline_pc = if execute_original {
                trampoline::create_original_trampoline(address.wrapping_add(4), original_opcode)?
            } else {
                0
            };

            unsafe {
                SLOTS[index] = InstrumentSlot {
                    used: true,
                    address,
                    original_opcode,
                    callback: Some(callback),
                    execute_original,
                    trampoline_pc,
                };
            }
            return Ok(());
        }
        index += 1;
    }

    Err(SigHookError::InstrumentSlotsFull)
}

pub(crate) unsafe fn original_opcode_by_address(address: u64) -> Option<u32> {
    let slot = unsafe { slot_by_address(address) }?;
    Some(slot.original_opcode)
}
