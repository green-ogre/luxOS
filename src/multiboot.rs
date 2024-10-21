#[repr(C)]
#[derive(Debug)]
pub struct MultibootHeader {
    pub flags: u32,
    pub mem_lower: u32,
    pub mem_upper: u32,
    pub boot_device: u32,
    pub cmdline: u32,
    pub mods_count: u32,
    pub mods_addr: u32,
    pub syms1: u32,
    pub syms2: u32,
    pub syms3: u32,
    pub syms4: u32,
    pub mmap_length: u32,
    pub mmap_addr: u32,
    pub drives_length: u32,
    pub drives_addr: u32,
    pub config_table: u32,
    pub boot_loader_name: u32,
    pub apm_table: u32,
    pub vbe_control_info: u32,
    pub vbe_mode_info: u32,
    pub vbe_mode: u16,
    pub vbe_interface_seg: u16,
    pub vbe_interface_off: u16,
    pub vbe_interface_len: u16,
    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,
    pub framebuffer_type: u8,
    pub color_info: [u8; 5],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VbeModeInfo {
    attributes: u16,
    window_a: u8,
    window_b: u8,
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32,
    pitch: u16,
    width: u16,
    height: u16,
    w_char: u8,
    y_char: u8,
    planes: u8,
    bpp: u8,
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved0: u8,
    red_mask: u8,
    red_position: u8,
    green_mask: u8,
    green_position: u8,
    blue_mask: u8,
    blue_position: u8,
    reserved_mask: u8,
    reserved_position: u8,
    direct_color_attributes: u8,
    framebuffer: u32,
    off_screen_mem_off: u32,
    off_screen_mem_size: u16,
    reserved1: [u8; 206],
}

pub fn verify_mutliboot_magic(magic: u32) {
    if magic != 0x2BADB002 {
        panic!("Invalid mutliloader magic");
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn parse_multiboot_header(magic: u32, _multiboot_header: &MultibootHeader) {
    verify_mutliboot_magic(magic);

    // VBE table is valid
    // assert_eq!(1, (_multiboot_header.flags >> 11) & 1);
    // crate::info!("{:#b}", _multiboot_header.vbe_mode);
    // let vbe_info = _multiboot_header.vbe_mode_info as *const u16 as *const VbeModeInfo;
    // crate::info!("{:#?}", unsafe { *vbe_info });

    // serial_println!("flags: {:#b}", multiboot_header.flags);
    // serial_println!(
    //     "mem_lower: {}, present: {}",
    //     multiboot_header.mem_lower,
    //     (multiboot_header.flags & (1 << 0)) > 0
    // );
    // serial_println!(
    //     "mem_upper: {}, present: {}",
    //     multiboot_header.mem_upper,
    //     (multiboot_header.flags & (1 << 0)) > 0
    // );
    // serial_println!(
    //     "boot_device: {}, present: {}",
    //     multiboot_header.boot_device,
    //     (multiboot_header.flags & (1 << 1)) > 0
    // );
    // serial_println!(
    //     "cmdline: {}, present: {}",
    //     multiboot_header.cmdline,
    //     (multiboot_header.flags & (1 << 2)) > 0
    // );
    // serial_println!(
    //     "mods_count: {}, present: {}",
    //     multiboot_header.mods_count,
    //     (multiboot_header.flags & (1 << 3)) > 0
    // );
    // serial_println!(
    //     "mods_addr: {}, present: {}",
    //     multiboot_header.mods_addr,
    //     (multiboot_header.flags & (1 << 3)) > 0
    // );
    // serial_println!(
    //     "syms1: {}, present: {}",
    //     multiboot_header.syms1,
    //     (multiboot_header.flags & (1 << 4)) > 0
    // );
    // serial_println!(
    //     "syms2: {}, present: {}",
    //     multiboot_header.syms2,
    //     (multiboot_header.flags & (1 << 4)) > 0
    // );
    // serial_println!(
    //     "syms3: {}, present: {}",
    //     multiboot_header.syms3,
    //     (multiboot_header.flags & (1 << 4)) > 0
    // );
    // serial_println!(
    //     "mmap_length: {}, present: {}",
    //     multiboot_header.mmap_length,
    //     (multiboot_header.flags & (1 << 6)) > 0
    // );
    // serial_println!(
    //     "mmap_addr: {}, present: {}",
    //     multiboot_header.mmap_addr,
    //     (multiboot_header.flags & (1 << 6)) > 0
    // );
    // serial_println!(
    //     "drives_length: {}, present: {}",
    //     multiboot_header.drives_length,
    //     (multiboot_header.flags & (1 << 7)) > 0
    // );
    // serial_println!(
    //     "drives_addr: {}, present: {}",
    //     multiboot_header.drives_addr,
    //     (multiboot_header.flags & (1 << 7)) > 0
    // );
    // serial_println!(
    //     "config_table: {}, present: {}",
    //     multiboot_header.config_table,
    //     (multiboot_header.flags & (1 << 8)) > 0
    // );
    // serial_println!(
    //     "boot_loader_name: {}, present: {}",
    //     multiboot_header.boot_loader_name,
    //     (multiboot_header.flags & (1 << 9)) > 0
    // );
    // serial_println!(
    //     "apm_table: {}, present: {}",
    //     multiboot_header.apm_table,
    //     (multiboot_header.flags & (1 << 10)) > 0
    // );

    // if (multiboot_header.flags & (1 << 9)) > 0 {
    //     let boot_loader_name = multiboot_header.boot_loader_name as *const u8;
    //     let name = unsafe { core::ffi::CStr::from_ptr(boot_loader_name as *const i8) };
    //     if let Ok(name_str) = name.to_str() {
    //         serial_println!("Boot loader name: {}", name_str);
    //     }
    // }
    //
    // let memory_range = multiboot_header.mem_upper - multiboot_header.mem_lower;
    // serial_println!("Memory range: {}mb", memory_range / 1024);
}
