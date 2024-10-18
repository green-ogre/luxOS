use alloc::{
    string::{String, ToString},
    vec::Vec,
};

pub fn get_cpu_features() -> Vec<CpuidFeatureEdx> {
    let b: u32;
    let d: u32;
    let c: u32;

    // Verify vendor id
    unsafe {
        core::arch::asm!(
            "mov eax, 0x0",
            "cpuid",
            out("ebx") b,
            out("edx") d,
            out("ecx") c,
        );
    }

    let mut vendor_id = String::new();

    let mut push_reg = |reg: u32| {
        vendor_id.push((reg & 0xFF) as u8 as char);
        vendor_id.push(((reg >> 8) & 0xFF) as u8 as char);
        vendor_id.push(((reg >> 16) & 0xFF) as u8 as char);
        vendor_id.push(((reg >> 24) & 0xFF) as u8 as char);
    };

    push_reg(b);
    push_reg(d);
    push_reg(c);

    // debug!("CPU vendor id: {}", vendor_id);
    assert_eq!("GenuineIntel".to_string(), vendor_id);

    // Gather cpu features
    let d: u32;
    unsafe {
        core::arch::asm!(
            "mov eax, 0x1",
            "cpuid",
            out("edx") d,
        );
    }

    let mut features: Vec<CpuidFeatureEdx> = Vec::with_capacity(32);
    // debug!("cpuid: {:#b}", d);
    for i in 0..32 {
        // TODO: match statement
        let feature = (d >> i) & 1;
        if feature != 0 {
            #[allow(clippy::missing_transmute_annotations)]
            features.push(unsafe { core::mem::transmute(1 << i) });
        }
    }
    // debug!("features: {:?}", features);
    features
}

#[repr(u32)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms, unused)]
/// Only found within newer chips?
enum CpuidFeatureEcx {
    SSE3 = 1 << 0,
    PCLMUL = 1 << 1,
    DTES64 = 1 << 2,
    MONITOR = 1 << 3,
    DS_CPL = 1 << 4,
    VMX = 1 << 5,
    SMX = 1 << 6,
    EST = 1 << 7,
    TM2 = 1 << 8,
    SSSE3 = 1 << 9,
    CID = 1 << 10,
    SDBG = 1 << 11,
    FMA = 1 << 12,
    CX16 = 1 << 13,
    XTPR = 1 << 14,
    PDCM = 1 << 15,
    PCID = 1 << 17,
    DCA = 1 << 18,
    SSE4_1 = 1 << 19,
    SSE4_2 = 1 << 20,
    X2APIC = 1 << 21,
    MOVBE = 1 << 22,
    POPCNT = 1 << 23,
    TSC = 1 << 24,
    AES = 1 << 25,
    XSAVE = 1 << 26,
    OSXSAVE = 1 << 27,
    AVX = 1 << 28,
    F16C = 1 << 29,
    RDRAND = 1 << 30,
    HYPERVISOR = 1 << 31,
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms, unused)]
pub enum CpuidFeatureEdx {
    FPU = 1 << 0,
    VME = 1 << 1,
    DE = 1 << 2,
    PSE = 1 << 3,
    TSC = 1 << 4,
    MSR = 1 << 5,
    PAE = 1 << 6,
    MCE = 1 << 7,
    CX8 = 1 << 8,
    APIC = 1 << 9,
    SEP = 1 << 11,
    MTRR = 1 << 12,
    PGE = 1 << 13,
    MCA = 1 << 14,
    CMOV = 1 << 15,
    PAT = 1 << 16,
    PSE36 = 1 << 17,
    PSN = 1 << 18,
    CLFLUSH = 1 << 19,
    DS = 1 << 21,
    ACPI = 1 << 22,
    MMX = 1 << 23,
    FXSR = 1 << 24,
    SSE = 1 << 25,
    SSE2 = 1 << 26,
    SS = 1 << 27,
    HTT = 1 << 28,
    TM = 1 << 29,
    IA64 = 1 << 30,
    PBE = 1 << 31,
}
