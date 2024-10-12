.set ALIGN,    1<<0
.set MEMINFO,  1<<1
.set MAGIC,    0x1BADB002
.set FLAGS,    ALIGN | MEMINFO
.set CHECKSUM, -(MAGIC + FLAGS)

.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM
.long 0, 0, 0, 0, 0  # unused
.long 0              # 0 = set graphics mode
.long 1024, 768, 32  # width, height, depth

.section .bss
.align 16
stack_bottom:
.skip 16384
stack_top:

.section .text
.global _start
.type _start, @function
_start:
  mov $stack_top, %esp

  # Multiboot info structure address
  pushl %ebx
  # Magic value
  pushl %eax

  call kmain

  cli
1:hlt
	jmp 1b

.size _start, . - _start
