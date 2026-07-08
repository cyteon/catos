OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

PHDRS
{
    text            PT_LOAD;
}

SECTIONS
{
    . = 0xffffffff80000000

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .text : { *(.text, .text.*) } :text

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}
