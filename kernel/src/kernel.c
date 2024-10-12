#include <stdint.h>

typedef uint32_t u32;

#define MULTIBOOT_BOOTLOADER_MAGIC 0xE85250D6

typedef struct {
  uint32_t flags;

  uint32_t mem_lower;
  uint32_t mem_upper;

  uint32_t boot_device;

  uint32_t cmdline;

  uint32_t mods_count;
  uint32_t mods_addr;

  uint32_t syms[4];

  uint32_t mmap_length;
  uint32_t mmap_addr;

  uint32_t drives_length;
  uint32_t drives_addr;

  uint32_t config_table;

  uint32_t boot_loader_name;

  uint32_t apm_table;

  uint32_t vbe_control_info;
  uint32_t vbe_mode_info;
  uint16_t vbe_mode;
  uint16_t vbe_interface_seg;
  uint16_t vbe_interface_off;
  uint16_t vbe_interface_len;

  uint64_t framebuffer_addr;
  uint32_t framebuffer_pitch;
  uint32_t framebuffer_width;
  uint32_t framebuffer_height;
  uint8_t framebuffer_bpp;
  uint8_t framebuffer_type;
  union {
    struct {
      uint32_t framebuffer_palette_addr;
      uint16_t framebuffer_palette_num_colors;
    };
    struct {
      uint8_t framebuffer_red_field_position;
      uint8_t framebuffer_red_mask_size;
      uint8_t framebuffer_green_field_position;
      uint8_t framebuffer_green_mask_size;
      uint8_t framebuffer_blue_field_position;
      uint8_t framebuffer_blue_mask_size;
    };
  };
} multiboot_info;

#define VGA_WIDTH 80
#define VGA_HEIGHT 25

enum vga_color {
  VGA_COLOR_BLACK = 0,
  VGA_COLOR_BLUE = 1,
  VGA_COLOR_GREEN = 2,
  VGA_COLOR_CYAN = 3,
  VGA_COLOR_RED = 4,
  VGA_COLOR_MAGENTA = 5,
  VGA_COLOR_BROWN = 6,
  VGA_COLOR_LIGHT_GREY = 7,
  VGA_COLOR_DARK_GREY = 8,
  VGA_COLOR_LIGHT_BLUE = 9,
  VGA_COLOR_LIGHT_GREEN = 10,
  VGA_COLOR_LIGHT_CYAN = 11,
  VGA_COLOR_LIGHT_RED = 12,
  VGA_COLOR_LIGHT_MAGENTA = 13,
  VGA_COLOR_LIGHT_BROWN = 14,
  VGA_COLOR_WHITE = 15,
};

static inline void write_vga_uc(uint16_t *buf, uint16_t offset, char uc,
                                uint8_t attribute) {
  *(buf + offset) = (uint16_t)uc | (uint16_t)attribute << 8;
}

static inline void write_vga_uc_str(uint16_t *buf, uint16_t offset,
                                    const char *uc_str, uint8_t attribute) {
  for (int i = 0; *uc_str; i++)
    write_vga_uc(buf, offset + i, *uc_str++, attribute);
}

static inline uint8_t blink(uint8_t attribute) { return attribute | 0x80; }

static inline void clear_screen(uint16_t *buf) {
  for (int i = 0; i < VGA_WIDTH * VGA_HEIGHT; i++)
    write_vga_uc(buf, i, ' ', VGA_COLOR_BLACK);
}

/*
 * TODO:
 *
 * If your kernel is booted by a Multiboot-compliant bootloader, like GRUB, you
 * are provided a memory map. You can set up the stack by looking for free
 * memory chunks of the appropriate size. You just have to ensure that you don't
 * overwrite any important data or code when setting the stack pointer.
 * https://wiki.osdev.org/Stack
 *
 * When debugging, a stack trace is often shown and can be helpful. Stack Trace
 * describes how this can be done and provides sample code for X86 CDECL using
 * the stack layout above.
 * https://wiki.osdev.org/Stack_Trace
 * */
void kmain(u32 magic, u32 addr) {
  // multiboot_info *mbi = (multiboot_info *)addr;

  // if (magic != MULTIBOOT_BOOTLOADER_MAGIC) {
  //   // Error: not loaded by a Multiboot-compliant bootloader
  //   return;
  // }
  //
  // if (mbi->framebuffer_addr) {
  //   uint32_t *pixel = (uint32_t *)(mbi->framebuffer_addr);
  //   *pixel = 0xffffffff;
  //   // printf("Framebuffer address: 0x%x\n", mbi->framebuffer_addr);
  //   // printf("Framebuffer dimensions: %dx%d, %d bpp\n",
  //   mbi->framebuffer_width,
  //   //        mbi->framebuffer_height, mbi->framebuffer_bpp);
  // }

  uint16_t *term_buf = (uint16_t *)0xB8000;
  clear_screen(term_buf);
  write_vga_uc_str(term_buf, 0, "Hello, World!", VGA_COLOR_LIGHT_RED);
}
