./iso.sh
if [ $? -ne 0 ]; then
    exit 1
fi

# qemu-system-x86_64 -cdrom kernel/build/lux.iso -m 512M \
#     -device qemu-xhci -M q35
qemu-system-i386 -kernel isodir/boot/kernel.bin -m 128M -vga std \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio
