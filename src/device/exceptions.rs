use crate::device;

pub fn check_pending_interrupts(device: &mut device::Device) {
    if (device.cpu.cop0.regs[device::cop0::COP0_STATUS_REG as usize]
        & device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize]
        & device::cop0::COP0_CAUSE_IP_MASK)
        == 0
    {
        // interrupt disabled, or no pending interrupts
        return;
    }

    if (device.cpu.cop0.regs[device::cop0::COP0_STATUS_REG as usize]
        & (device::cop0::COP0_STATUS_IE
            | device::cop0::COP0_STATUS_EXL
            | device::cop0::COP0_STATUS_ERL))
        != device::cop0::COP0_STATUS_IE
    {
        // interrupts disabled globally, or error/exception is already set
        return;
    }

    exception_general(device);
}

pub fn floating_point_exception(device: &mut device::Device) {
    device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
        device::cop0::COP0_CAUSE_EXCCODE_FPE;

    exception_general(device)
}

pub fn trap_exception(device: &mut device::Device) {
    device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
        device::cop0::COP0_CAUSE_EXCCODE_TR;

    exception_general(device)
}

pub fn syscall_exception(device: &mut device::Device) {
    device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
        device::cop0::COP0_CAUSE_EXCCODE_SYS;

    exception_general(device)
}

pub fn break_exception(device: &mut device::Device) {
    device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
        device::cop0::COP0_CAUSE_EXCCODE_BP;

    exception_general(device)
}

pub fn reserved_exception(device: &mut device::Device, cop: u64) {
    device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
        device::cop0::COP0_CAUSE_EXCCODE_RI | cop;

    exception_general(device)
}

pub fn cop_unusable_exception(device: &mut device::Device, cop: u64) {
    device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
        device::cop0::COP0_CAUSE_EXCCODE_CPU | cop;

    exception_general(device)
}

pub fn tlb_miss_exception(
    device: &mut device::Device,
    address: u64,
    access_type: device::memory::AccessType,
) {
    if access_type == device::memory::AccessType::Read {
        device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
            device::cop0::COP0_CAUSE_EXCCODE_TLBL
    } else {
        device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] =
            device::cop0::COP0_CAUSE_EXCCODE_TLBS
    }

    device.cpu.cop0.regs[device::cop0::COP0_BADVADDR_REG as usize] = address;
    device::memory::masked_write_64(
        &mut device.cpu.cop0.regs[device::cop0::COP0_CONTEXT_REG as usize],
        address >> 9,
        device::cop0::COP0_CONTEXT_BADVPN2_MASK,
    );
    device::memory::masked_write_64(
        &mut device.cpu.cop0.regs[device::cop0::COP0_XCONTEXT_REG as usize],
        address >> 9,
        device::cop0::COP0_XCONTEXT_BADVPN2_MASK,
    );
    device::memory::masked_write_64(
        &mut device.cpu.cop0.regs[device::cop0::COP0_XCONTEXT_REG as usize],
        address >> 31,
        device::cop0::COP0_XCONTEXT_REGION_MASK,
    );

    if device.cpu.cop0.regs[device::cop0::COP0_STATUS_REG as usize] & device::cop0::COP0_STATUS_EXL
        == 0
    {
        device.cpu.pc -= 0x180
    }

    exception_general(device)
}

pub fn exception_general(device: &mut device::Device) {
    if device.cpu.cop0.regs[device::cop0::COP0_STATUS_REG as usize] & device::cop0::COP0_STATUS_EXL
        == 0
    {
        device.cpu.cop0.regs[device::cop0::COP0_EPC_REG as usize] = device.cpu.pc;
        if device::cpu::in_delay_slot(device) {
            device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] |=
                device::cop0::COP0_CAUSE_BD;
            device.cpu.cop0.regs[device::cop0::COP0_EPC_REG as usize] -= 4;
        } else {
            device.cpu.cop0.regs[device::cop0::COP0_CAUSE_REG as usize] &=
                !device::cop0::COP0_CAUSE_BD;
        }
    }

    device.cpu.cop0.regs[device::cop0::COP0_STATUS_REG as usize] |= device::cop0::COP0_STATUS_EXL;

    let vector_offset: u32 = 0x180;
    if device.cpu.cop0.regs[device::cop0::COP0_STATUS_REG as usize] & device::cop0::COP0_STATUS_BEV
        == 0
    {
        device.cpu.pc = device::cpu_instructions::se32((0x80000000 + vector_offset) as i32);
    } else {
        device.cpu.pc = device::cpu_instructions::se32((0xBFC00200 + vector_offset) as i32);
    }

    device.cpu.branch_state.state = device::cpu::State::Exception;

    device::cop0::add_cycles(device, 2);
}
