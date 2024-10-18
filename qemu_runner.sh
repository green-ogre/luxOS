#!/bin/sh

target="$1"

mkdir -p isodir
mkdir -p isodir/boot
mkdir -p isodir/boot/grub

cp $target isodir/boot/kernel.bin
cat >isodir/boot/grub/grub.cfg <<EOF
menuentry "Lux" {
	multiboot /boot/kernel.bin
}
EOF
grub-mkrescue -o lux.iso isodir >/dev/null 2>&1

qemu-system-i386 -m 512M -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -cdrom lux.iso
# qemu-system-i386 -vga std -cpu host -enable-kvm -m 512M -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -kernel $target

exit $(($? & ~33))
