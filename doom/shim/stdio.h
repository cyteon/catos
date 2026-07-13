#ifndef SHIM_STDIO_H
#define SHIM_STDIO_H

#include <stdarg.h>
#include <stddef.h>

typedef struct FILE FILE;

#define EOF (-1)
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

extern FILE* stdout;
extern FILE* stderr;

FILE* fopen(const char* filename, const char* mode);

int fclose(FILE* file);
size_t fread(void* buf, size_t size, size_t count, FILE* file);
size_t fwrite(const void* buf, size_t size, size_t count, FILE* file);

int fseek(FILE* file, long offset, int whence);
long ftell(FILE* file);
int fflush(FILE* file);

int printf(const char* format, ...);
int fprintf(FILE* file, const char* format, ...);
int sprintf(char* str, const char* format, ...);
int snprintf(char* str, size_t size, const char* format, ...);
int vfprintf(FILE *file, const char* format, va_list args);
int vsnprintf(char* str, size_t size, const char* format, va_list args);
int sscanf(const char* str, const char* format, ...);

int puts(const char* str);
int putchar(int c);

int remove(const char* filename);
int rename(const char* old_filename, const char* new_filename);

#endif
