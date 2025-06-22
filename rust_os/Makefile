# Makefile for the Rust OS project with Docker-based ISO creation

#==== Project Settings ====#
NAME := rust_os
ARCH := i386

#==== Directory Structure ====#
SRC_DIR     := src
ASM_DIR     := $(SRC_DIR)/boot
CONFIG_DIR  := config
BUILD_DIR   := build
BUILD_OBJ   := $(BUILD_DIR)/obj
BUILD_BIN   := $(BUILD_DIR)/bin
ISO_DIR     := iso
DIST_DIR    := dist
TOOLS_DIR   := tools

#==== Source Files ====#
BOOT_SRC    := $(ASM_DIR)/boot.asm
KERNEL_SRC  := $(SRC_DIR)/*.rs
LINKER      := $(CONFIG_DIR)/linker.ld
TARGET_SPEC := $(CONFIG_DIR)/$(ARCH)-unknown-none.json
GRUB_CFG    := $(CONFIG_DIR)/grub/grub.cfg

#==== Output Files ====#
BOOT_OBJ    := $(BUILD_OBJ)/boot.o
KERNEL_OBJ  := $(BUILD_OBJ)/kernel.o
KERNEL_BIN  := $(BUILD_BIN)/$(NAME).bin
ISO_FILE    := $(DIST_DIR)/$(NAME).iso

#==== ISO Structure ====#
ISO_BOOT    := $(ISO_DIR)/boot
ISO_GRUB    := $(ISO_BOOT)/grub
ISO_KERNEL  := $(ISO_BOOT)/$(NAME).bin
ISO_CFG     := $(ISO_GRUB)/grub.cfg

#==== Tools and Flags ====#
NASM        := nasm
NASMFLAGS   := -f elf32
LD          := ld
LDFLAGS     := -m elf_i386 -T $(LINKER)
CARGO       := cargo +nightly
CARGO_FLAGS := --target $(TARGET_SPEC) --release
QEMU        := qemu-system-i386
QEMU_FLAGS  := -monitor stdio

#==== Docker Settings ====#
DOCKER_CONTAINER_NAME_BIN := bin_builder
DOCKER_CONTAINER_NAME_ISO := iso_builder
# *** .bin file *** #
DOCKER_BIN_IMAGE := $(NAME)-buildenv
DOCKER_BIN_DIR := $(TOOLS_DIR)/bin/env
DOCKER_BIN_CMD := docker run \
  --name $(DOCKER_CONTAINER_NAME_BIN) --rm \
  --platform linux/amd64 \
  -v $(CURDIR):/project \
  -w /project \
  $(DOCKER_BIN_IMAGE)
# *** .iso file *** #
DOCKER_ISO_IMAGE := $(NAME)-isoenv
DOCKER_ISO_DIR := $(TOOLS_DIR)/iso/env
DOCKER_ISO_CMD := docker run \
  --name $(DOCKER_CONTAINER_NAME_ISO) --rm \
  --platform linux/amd64 \
  -v $(CURDIR):/project \
  -w /project \
  $(DOCKER_ISO_IMAGE)

#==== Build Targets ====#
.PHONY: all kernel clean clean-all iso run run-iso dirs re docker-image

# Main targets
all: dirs kernel

kernel: docker-bin-image $(KERNEL_BIN)

# Directory creation
dirs: $(BUILD_OBJ) $(BUILD_BIN) $(DIST_DIR)

$(BUILD_OBJ) $(BUILD_BIN) $(DIST_DIR):
	@mkdir -p $@

# Main kernel build
$(KERNEL_BIN): $(BOOT_OBJ) $(KERNEL_OBJ) $(LINKER) | $(BUILD_BIN)
	@echo "==> Linking kernel binary"
	$(DOCKER_BIN_CMD) $(LD) $(LDFLAGS) -o $@ $(BOOT_OBJ) $(KERNEL_OBJ)
	@echo "==> Kernel binary created at $@"

# Assembly compilation
$(BOOT_OBJ): $(BOOT_SRC) docker-bin-image | $(BUILD_OBJ)
	@echo "==> Assembling boot code using Docker"
	$(DOCKER_BIN_CMD) nasm $(NASMFLAGS) $(BOOT_SRC) -o $(BOOT_OBJ)
	@echo "==> Boot object created at $@"

# Rust compilation
$(KERNEL_OBJ): $(KERNEL_SRC) $(TARGET_SPEC) | $(BUILD_OBJ)
	@echo "==> Compiling Rust kernel"
	$(DOCKER_BIN_CMD) cargo +nightly build --target $(TARGET_SPEC) --release
	$(DOCKER_BIN_CMD) cp target/$(ARCH)-unknown-none/release/lib$(NAME).a $@
	@echo "==> Kernel object created at $@"

#==== Build Docker Images ====#
docker-bin-image:
	@echo "==> Building Docker image for .bin creation"
	docker build $(DOCKER_BIN_DIR) --platform linux/amd64 -t $(DOCKER_BIN_IMAGE)

docker-iso-image:
	@echo "==> Building Docker image for ISO creation"
	docker build $(DOCKER_ISO_DIR) --platform linux/amd64 -t $(DOCKER_ISO_IMAGE)

# #==== ISO Creation (Docker-based) ====#
# .PHONY: iso
# iso: kernel docker-iso-image
# 	@echo "==> Creating ISO using Docker container"
# 	@echo "==> Preparing ISO structure"
# 	@mkdir -p $(ISO_GRUB)
# 	@cp $(KERNEL_BIN) $(ISO_KERNEL)
# 	@cp $(GRUB_CFG) $(ISO_CFG)
# 	@mkdir -p dist
# 	@echo "==> Generating bootable ISO in Docker"
# 	$(DOCKER_CMD) bash -c "cd /project && grub-mkrescue -o $(ISO_FILE) $(ISO_DIR)"
# 	@echo "==> ISO created at $(ISO_FILE)"

#==== ISO Creation (Docker-based) ====#
.PHONY: iso
iso: kernel docker-iso-image
	@echo "==> Creating ISO using Docker container"
	@echo "==> Preparing ISO structure"
	@mkdir -p $(ISO_GRUB)
	@cp $(KERNEL_BIN) $(ISO_KERNEL)
	@cp $(GRUB_CFG) $(ISO_CFG)
	@mkdir -p dist
	@echo "==> Generating bootable ISO in Docker"
	$(DOCKER_ISO_CMD) bash -c "cd /project && grub-mkrescue -o $(ISO_FILE) $(ISO_DIR)"
	@echo "==> ISO created at $(ISO_FILE)"

#==== Running and Testing ====#
run: $(KERNEL_BIN)
	@echo "==> Running kernel directly with QEMU"
	$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL_BIN)

run-iso: iso
	@echo "==> Running from bootable ISO with QEMU"
	$(QEMU) $(QEMU_FLAGS) -cdrom $(ISO_FILE)

.PHONY: stop-containers
stop-containers:
	@echo "Stopping containers (if running)..."
	-docker stop $(DOCKER_CONTAINER_NAME_BIN) 2>/dev/null || true
	-docker stop $(DOCKER_CONTAINER_NAME_ISO) 2>/dev/null || true

.PHONY: remove-containers
remove-containers: stop-containers
	@echo "Removing containers (if they exist)..."
	-docker rm $(DOCKER_CONTAINER_NAME_BIN) 2>/dev/null || true
	-docker rm $(DOCKER_CONTAINER_NAME_ISO) 2>/dev/null || true

.PHONY: clean-images
clean-images: remove-containers
	@echo "Removing Docker images..."
	-docker image rm $(DOCKER_BIN_IMAGE) 2>/dev/null || true
	-docker image rm $(DOCKER_ISO_IMAGE) 2>/dev/null || true

#==== Cleaning ====#
clean: clean-images
	@echo "==> Cleaning build artifacts"
	@rm -rf $(BUILD_DIR) $(ISO_DIR)
	@echo "==> Build directories cleaned"

clean-all: clean
	@echo "==> Cleaning all generated files"
	@rm -rf $(DIST_DIR)
	@echo "==> Distribution directory cleaned"

re: clean all
	@echo "==> Project rebuilt from scratch"

#==== Help Section ====#
.PHONY: help kernel run run-iso clean clean-all re docker-image

help:
	@echo "Rust OS Build System"
	@echo "===================="
	@echo "Available targets:"
	@echo "  all       	: Build the kernel binary"
	@echo "  kernel    	: Same as 'all'"
	@echo "  iso       	: Create a bootable ISO image using Docker"
	@echo "  run       	: Run the kernel directly in QEMU"
	@echo "  run-iso   	: Create ISO in Docker and boot it with QEMU"
	@echo "  clean     	: Remove build artifacts"
	@echo "  clean-all 	: Remove all generated files"
	@echo "  re        	: Rebuild everything from scratch"
	@echo "  docker-image  : Build the Docker image for ISO creation"
	@echo "  help      	: Show this help message"
	@echo ""
	@echo "Default target: all"