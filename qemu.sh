./iso.sh
if [ $? -ne 0 ]; then
  exit 1
fi

qemu-system-i386 -cdrom kernel/build/lux.iso -m 512M
# qemu-system-i386 -kernel isodir/boot/kernel.bin -m 128M -vga std
