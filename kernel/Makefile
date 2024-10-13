CC := i686-elf-gcc
CFLAGS := -Wall -Wextra -O2 -ffreestanding -nostdlib
CPPFLAGS := -I../lib/include -Iinclude

TARGET := kernel.bin
SRC_DIR := src
BUILD_DIR := build

SRC_FILES := $(wildcard $(SRC_DIR)/*.c)
OBJ_FILES := $(patsubst $(SRC_DIR)/%.c,$(BUILD_DIR)/%.o,$(SRC_FILES))
LIB_OBJ_FILES := $(wildcard ../lib/build/*.o)

.PHONY: build_kernel
build_kernel: $(TARGET)

$(BUILD_DIR):
	mkdir -p $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c | $(BUILD_DIR)
	i686-elf-as src/i686_boot.s -o build/i686_boot.o
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ_FILES) | $(BUILD_DIR)
	$(CC) $(CFLAGS) -T linker.ld -o $(BUILD_DIR)/$(TARGET) $(OBJ_FILES) $(LIB_OBJ_FILES) build/i686_boot.o -lgcc

clean:
	rm -rf $(BUILD_DIR)
