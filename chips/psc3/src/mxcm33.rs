use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// MXCM33-0/1
    Mxcm33Registers {
        /// Control
        (0x000 => cm33_ctl: ReadWrite<u32, CM33_CTL::Register>),
        /// Command
        (0x004 => cm33_cmd: ReadWrite<u32, CM33_CMD::Register>),
        /// Status
        (0x008 => cm33_status: ReadWrite<u32, CM33_STATUS::Register>),
        (0x00C => _reserved0),
        /// CM33 interrupt status
        (0x040 => cm33_int_status: [ReadWrite<u32, CM33_INT_STATUS::Register>; 16]),
        /// CM33 NMI control
        (0x080 => cm33_nmi_ctl: [ReadWrite<u32, CM33_NMI_CTL::Register>; 4]),
        (0x090 => _reserved1),
        /// CM33 event control
        (0x0C0 => cm33_event_ctl: ReadWrite<u32>),
        (0x0C4 => _reserved2),
        /// CM33 secure vector table base
        (0x1000 => cm33_s_vector_table_base: ReadWrite<u32, CM33_VECTOR_TABLE_BASE::Register>),
        /// CM33 non-secure vector table base
        (0x1004 => cm33_ns_vector_table_base: ReadWrite<u32, CM33_VECTOR_TABLE_BASE::Register>),
        (0x1008 => _reserved3),
        /// CM33 protection context control
        (0x2000 => cm33_pc_ctl: ReadWrite<u32>),
        (0x2004 => _reserved4),
        /// CM33 protection context 0 handler
        /// Address of the protection context 0 handler. This field is used to detect entry to Cypress 'trusted' code through an exception/interrupt.
        (0x2040 => cm33_pc0_handler: ReadWrite<u32>),
        (0x2044 => _reserved5),
        /// CM33 protection context 1 handler
        /// Address of the protection context 0 handler.
        (0x2100 => cm33_pc1_handler: ReadWrite<u32>),
        (0x2104 => _reserved6),
        /// CM33 protection context 2 handler
        /// Address of the protection context 0 handler.
        (0x2140 => cm33_pc2_handler: ReadWrite<u32>),
        (0x2144 => _reserved7),
        /// CM33 protection context 3 handler
        /// Address of the protection context 0 handler.
        (0x2180 => cm33_pc3_handler: ReadWrite<u32>),
        (0x2184 => _reserved8),
        /// CM33 system interrupt control
        (0x8000 => cm33_system_int_ctl: [ReadWrite<u32, CM33_SYSTEM_INT_CTL::Register>; 1023]),
        (0x8FFC => @END),
    }
}
register_bitfields![u32,
CM33_CTL [
    /// N/A
    CPU_WAIT OFFSET(4) NUMBITS(1) [],
    /// N/A
    LOCKNSVTOR OFFSET(8) NUMBITS(1) [],
    /// N/A
    LOCKSVTAIRCR OFFSET(9) NUMBITS(1) [],
    /// N/A
    LOCKSMPU OFFSET(10) NUMBITS(1) [],
    /// N/A
    LOCKNSMPU OFFSET(11) NUMBITS(1) [],
    /// N/A
    LOCKSAU OFFSET(12) NUMBITS(1) [],
    /// CPU floating point unit (FPU) exception mask for the CPU's FPCSR.IOC 'invalid operation' exception condition:
    /// '0': The CPU's exception condition does NOT activate the CPU's floating point interrupt.
    /// '1': the CPU's exception condition activates the CPU's floating point interrupt.
    ///
    /// Note: the ARM architecture does NOT support FPU exceptions; i.e. there is no precise FPU exception handler. Instead, FPU conditions are captured in the CPU's FPCSR register and the conditions are provided as CPU interface signals. The interface signals are 'masked' with the fields provided by this register (CM33_CTL). The 'masked' signals are reduced/OR-ed into a single CPU floating point interrupt signal. The associated CPU interrupt handler allows for imprecise handling of FPU exception conditions.
    ///
    /// Note: the CPU's FPCSR exception conditions are 'sticky'. Typically, the CPU FPU interrupt handler will clear the exception condition(s) to '0'.
    ///
    /// Note: by default, the FPU exception masks are '0'. Therefore, FPU exception conditions will NOT activate the CPU's floating point interrupt.
    IOC_MASK OFFSET(24) NUMBITS(1) [],
    /// N/A
    DZC_MASK OFFSET(25) NUMBITS(1) [],
    /// N/A
    OFC_MASK OFFSET(26) NUMBITS(1) [],
    /// N/A
    UFC_MASK OFFSET(27) NUMBITS(1) [],
    /// N/A
    IXC_MASK OFFSET(28) NUMBITS(1) [],
    /// N/A
    IDC_MASK OFFSET(31) NUMBITS(1) []
],
CM33_CMD [
    /// Processor enable:
    /// '0': Disabled. Processor clock is turned off and reset is activated. After SW clears this field to '0', HW automatically sets this field to '1'. This effectively results in a CM33 reset, followed by a CM33 warm boot.
    /// '1': Enabled.
    /// Note: The intent is that this bit is modified only through an external probe or by the other CM33 while this CM33 is in Sleep or DeepSleep power mode. If this field is cleared to '0' by this CM33 itself, it should be done under controlled conditions (such that undesirable side effects can be prevented).
    ///
    /// Note: The CM33 CPU has a AIRCR.SYSRESETREQ register field that allows the CM33 to reset the complete device (ENABLED only disables/enables the CM33), resulting in a warm boot. This CPU register field has similar 'built-in protection' as this register to prevent accidental system writes (the upper 16-bits of the register need to be written with a 0x05fa key value; see CPU user manual for more details).
    ENABLED OFFSET(1) NUMBITS(1) [],
    /// Register key (to prevent accidental writes).
    /// - Should be written with a 0x05fa key value for the write to take effect.
    /// - Always reads as 0xfa05.
    VECTKEYSTAT OFFSET(16) NUMBITS(16) []
],
CM33_STATUS [
    /// Specifies if the CPU is in Active, Sleep or DeepSleep power mode:
    /// - Active power mode: SLEEPING is '0'.
    /// - Sleep power mode: SLEEPING is '1' and SLEEPDEEP is '0'.
    /// - DeepSleep power mode: SLEEPING is '1' and SLEEPDEEP is '1'.
    SLEEPING OFFSET(0) NUMBITS(1) [],
    /// Specifies if the CPU is in Sleep or DeepSleep power mode. See SLEEPING field.
    SLEEPDEEP OFFSET(1) NUMBITS(1) []
],
CM33_INT_STATUS [
    /// Lowest CM33 activated system interrupt index for given CPU interrupt.
    ///
    /// Multiple system interrupts can be mapped on the same CPU interrupt. The selected system interrupt is the system interrupt with the lowest system interrupt index that has an activated interrupt request at the time of the fetch (system_interrupts[SYSTEM_INT_IDX] is '1').
    ///
    /// The CPU interrupt handler SW can read SYSTEM_INT_IDX to determine the system interrupt that activated the handler.
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) [],
    /// Valid indication for SYSTEM_INT_IDX. When '0', no system interrupt for CPU interrupt 0 is valid/activated.
    SYSTEM_INT_VALID OFFSET(31) NUMBITS(1) []
],
CM33_NMI_CTL [
    /// System interrupt select for CPU NMI. The reset value ('1023') ensures that the CPU NMI is NOT connected to any system interrupt after DeepSleep reset.
    SYSTEM_INT_IDX OFFSET(0) NUMBITS(10) []
],
CM33_EVENT_CTL [
    /// One mask bit for each CPU (other than itself).
    /// 0: Mask is not set.
    /// 1: RX event from the corresponding CPU is masked.
    /// Bit 0: Other CM33 CPU event
    /// Bit 1: CM55_0 CPU event
    /// Bit 2: CM55_1 CPU event
    /// Bit 3: CM55_2 CPU event
    /// Bit 4: CM55_3 CPU event
    MASK OFFSET(0) NUMBITS(5) []
],
CM33_VECTOR_TABLE_BASE [
    /// Address of CM33 secure vector table to be used at reset.
    /// Default value:
    /// ADDR25: 0x0200000 if PROM is present (points to Secure ROM start address i.e. 0x1000_0000).
    /// ADDR25: 0x0680000 if PROM is not present (points to Secure RAMC0 start address i.e. 0x3400_0000).
    ADDR25 OFFSET(7) NUMBITS(25) []
],
CM33_PC_CTL [
    /// Valid fields for the protection context handler CM33_PCi_HANDLER registers:
    /// Bit 0: Valid field for CM33_PC0_HANDLER.
    /// Bit 1: Valid field for CM33_PC1_HANDLER.
    /// Bit 2: Valid field for CM33_PC2_HANDLER.
    /// Bit 3: Valid field for CM33_PC3_HANDLER.
    VALID OFFSET(0) NUMBITS(4) []
],
CM33_SYSTEM_INT_CTL [
    /// CPU interrupt index (legal range [0, 15]). This field specifies to which CPU interrupt the system interrupt is mapped. E.g., if CPU_INT_IDX is '6', the system interrupt is mapped to CPU interrupt '6'.
    ///
    /// Note: it is possible to map multiple system interrupts to the same CPU interrupt. It is advised to assign different priorities to the CPU interrupts and to assign system interrupts to CPU interrupts accordingly.
    ///
    /// Note: CPU_INT_IDX register width is derived from mxcm33 parameter IRQ_IDX_WIDTH.
    /// IRQ_IDX_WIDTH  = (CM33_INT_NR == 16) ? 4 : 3
    CPU_INT_IDX OFFSET(0) NUMBITS(4) [],
    /// Interrupt enable:
    /// '0': Disabled. The system interrupt will NOT be mapped to any CPU interrupt.
    /// '1': Enabled. The system interrupt is mapped on CPU interrupt CPU_INT_IDX.
    ///
    /// Note: the CPUs have dedicated XXX_SYSTEM_INT_CTL registers. In other words, the CPUs can use different CPU interrupts for the same system interrupt. However, typically only one of the CPUs will have the ENABLED field of a specific system interrupt set to '1'.
    CPU_INT_VALID OFFSET(31) NUMBITS(1) []
],
];
const MXCM33_BASE: StaticRef<Mxcm33Registers> =
    unsafe { StaticRef::new(0x42160000 as *const Mxcm33Registers) };

pub fn set_ns_vector_table_base(addr: u32) {
    MXCM33_BASE
        .cm33_ns_vector_table_base
        .write(CM33_VECTOR_TABLE_BASE::ADDR25.val(addr >> 7));
}
