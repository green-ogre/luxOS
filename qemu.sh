./iso.sh
# qemu-system-i386 -cdrom kernel/build/lux.iso
qemu-system-i386 -kernel isodir/boot/kernel.bin -m 128M -vga std
