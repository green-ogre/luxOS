#include <kernel/terminal.h>
#include <sint.h>

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

static inline void vga_write_uc(u16 *buf, u16 offset, char uc, u8 attribute) {
  *(buf + offset) = (u16)uc | (u16)attribute << 8;
}

static inline void vga_clear_screen(u16 *buf) {
  for (int i = 0; i < VGA_WIDTH * VGA_HEIGHT; i++)
    vga_write_uc(buf, i, ' ', VGA_COLOR_BLACK);
}

static uint16_t *term_buf = (uint16_t *)0xB8000;
static int offset = 0;

inline void clear_screen() { vga_clear_screen(term_buf); }

inline int terminal_writen(const char *buf, int n) {
  for (int i = 0; i < n; i++) {
    vga_write_uc(term_buf, offset++, *(buf + i), VGA_COLOR_WHITE);
  }

  return n;
}
