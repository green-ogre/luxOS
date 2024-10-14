#!/bin/sh

./iso.sh "$1"
# qemu-system-i386 -m 512M -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -cdrom lux.iso
qemu-system-i386 -m 512M -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -kernel "$1"

exit $(($? & ~33))
