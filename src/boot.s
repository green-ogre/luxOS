.set MAGIC,     0x1badb002
.set FLAGS,     7
.set CHECKSUM,  -(MAGIC + FLAGS)
.set MODE_TYPE, 0
.set WIDTH,     1024
.set HEIGHT,    768
.set DEPTH,     32

.set HEADER_ADDR,   0
.set LOAD_ADDR,     0
.set LOAD_END_ADDR, 0
.set BSS_END_ADDR,  0
.set ENTRY_ADDR,    0


# https://www.gnu.org/software/grub/manual/multiboot/multiboot.html#OS-image-format 
.section .multiboot
.long MAGIC
.long FLAGS
.long CHECKSUM
.long HEADER_ADDR
.long LOAD_ADDR
.long LOAD_END_ADDR
.long BSS_END_ADDR
.long ENTRY_ADDR
.long MODE_TYPE
.long WIDTH
.long HEIGHT
.long DEPTH
/* enough space for the returned header */
.space 4 * 13

.section .text
.global _start
.type _start, @function
_start:
    mov stack_top, esp

    push ebx
    push eax
    call kernel_main

    mov eax, 0x10
    out 0xf4, eax

_stop:
    cli
    hlt
    jmp _stop

.section .bss
.align 16

gdtr:
.skip 8
gdtr_code:
.skip 8
gdtr_data:
.skip 8

stack_bottom:
.skip 16384
stack_top:
.skip 4


