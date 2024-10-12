all: build_kernel

build_kernel:
	@i686-elf-as src/i686_boot.s -o build/i686_boot.o
	@i686-elf-gcc -c src/kernel.c -o build/kernel.o -ffreestanding -O2 -Wall -Wextra
	@i686-elf-gcc -T linker.ld -o build/kernel.bin -ffreestanding -O2 -nostdlib build/i686_boot.o build/kernel.o -lgcc

clean:
	@rm -f build/*
