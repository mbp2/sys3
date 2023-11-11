boot_source_files := $(shell find lib/boot -name *.c)
boot_object_files := $(patsubst lib/boot/%.c, build/boot/%.o, $(boot_source_files))

libc_source_files := $(shell find lib/libc -name *.c)
libc_object_files := $(patsubst lib/libc/%.c, build/libc/%.o, $(libc_source_files))

kernel_source_files := $(shell find main -name *.c)
kernel_object_files := $(patsubst main/%.c, build/kernel/%.o, $(kernel_source_files))

x86_64_asm_source_files := $(shell find lib/boot/asm -name *.asm)
x86_64_asm_object_files := $(patsubst lib/boot/asm/%.asm, build/boot/asm/%.o, $(x86_64_asm_source_files))

x86_64_object_files := $(boot_object_files) $(x86_64_asm_object_files)

$(kernel_object_files): build/kernel/%.o : main/%.c
	mkdir -p $(dir $@) && \
	x86_64-elf-gcc -c -I include -ffreestanding $(patsubst build/kernel/%.o, main/%.c, $@) -o $@

$(boot_object_files): build/boot/%.o : lib/boot/%.c
	mkdir -p $(dir $@) && \
	x86_64-elf-gcc -c -I include -ffreestanding $(patsubst build/boot/%.o, lib/boot/%.c, $@) -o $@

$(x86_64_asm_object_files): build/boot/asm/%.o : lib/boot/asm/%.asm
	mkdir -p $(dir $@) && \
	x86_64-elf-as -f elf64 $(patsubst build/boot/asm/%.o, lib/boot/asm/%.asm, $@) -o $@

$(libc_object_files): build/libc/%.o : lib/libc/%.c
	mkdir -p $(dir $@) && \
	x86_64-elf-gcc -c -I include -ffreestanding $(patsubst build/libc/%.o, lib/libc/%.c, $@) -o $@

.PHONY: build-x86_64
build-x86_64: $(kernel_object_files) $(libc_object_files) $(x86_64_object_files)
	mkdir -p dist/x86_64/boot && \
	x86_64-elf-ld -n -o dist/x86_64/kernel.bin -T lib/link/x86.link.ld $(kernel_object_files) $(x86_64_object_files) $(libc_object_files) && \
	cp dist/x86_64/kernel.bin boot/kernel.bin && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso boot
