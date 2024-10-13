CC := i686-elf-gcc
CFLAGS := -Wall -Wextra -O2 -ffreestanding -nostdlib
CPPFLAGS := -Iinclude -I../kernel/include

SRC_DIR := src
BUILD_DIR := build

SRC_FILES := $(wildcard $(SRC_DIR)/*.c)
OBJ_FILES := $(patsubst $(SRC_DIR)/%.c,$(BUILD_DIR)/%.o,$(SRC_FILES))

.PHONY: build_lib
build_lib: $(OBJ_FILES)

$(BUILD_DIR):
	mkdir -p $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(BUILD_DIR)/*

