#!/bin/sh
set -e

KERNEL="$1"
ISO_ROOT=target/iso_root
ISO=target/catos.iso

if [ ! -d target/limine ]; then
    git clone https://github.com/limine-bootloader/limine.git --branch=v9.x-binary --depth=1 target/limine
    make -C target/limine
fi

rm -rf "$ISO_ROOT"
mkdir -p "$ISO_ROOT/boot/limine" "$ISO_ROOT/EFI/BOOT"

cp "$KERNEL" "$ISO_ROOT/boot/kernel"
cp limine.conf "$ISO_ROOT/boot/limine/"
cp target/limine/limine-bios.sys \
   target/limine/limine-bios-cd.bin \
   target/limine/limine-uefi-cd.bin "$ISO_ROOT/boot/limine/"
cp target/limine/BOOTX64.EFI "$ISO_ROOT/EFI/BOOT/"

xorriso -as mkisofs -R -r -J \
    -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
        "$ISO_ROOT" -o "$ISO"

./target/limine/limine bios-install "$ISO"

qemu-system-x86_64 -M q35 -m 512M -cdrom "$ISO" -serial stdio -no-reboot -no-shutdown -d int,cpu_reset -display gtk
