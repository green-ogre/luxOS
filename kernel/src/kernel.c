#include <kernel/terminal.h>
#include <sint.h>
#include <sio.h>
#include <slib.h>
#include <sstring.h>

typedef struct {
    u32 flags;
    u32 mem_lower;
    u32 mem_upper;
    u32 boot_device;
    u32 cmdline;
    u32 mods_count;
    u32 mods_addr;
    u32 syms1;
    u32 syms2;
    u32 syms3;
    u32 mmap_length;
    u32 mmap_addr;
    u32 drives_length;
    u32 drives_addr;
    u32 config_table;
    u32 boot_loader_name;
    u32 apm_table;
    u32 vbe_control_info;
    u32 vbe_mode_info;
    u16 vbe_mode;
    u16 vbe_interface_seg;
    u32 vbe_interface_off;
    u32 vbe_interface_len;
    u32 framebuffer_addr;
    u32 framebuffer_pitch;
    u32 framebuffer_width;
    u32 framebuffer_height;
    u8 framebuffer_bpp;
    u8 framebuffer_type;
    u8 color_info[5];
} multiboot_header;

#define MULTIBOOT_BOOTLOADER_MAGIC 0x2BADB002

// typedef struct {
//     u32 width, height;
//     u32 addr;
// }

void kmain(u32 stack_ptr, const multiboot_header *mb_hdr, u32 magic)
{
    if (magic != MULTIBOOT_BOOTLOADER_MAGIC) {
        abort();
    }

    for (u32 i = 0; i < mb_hdr->framebuffer_width * mb_hdr->framebuffer_height;
         i++)
        *((u32 *)mb_hdr->framebuffer_addr + i) = 0x000000;
}
