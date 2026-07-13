#include <stdarg.h>
#include <stddef.h>

typedef struct FILE FILE;

int vprintf_(const char* fmt, va_list ap);
int vsnprintf_(char* s, size_t n, const char* fmt, va_list ap);

int printf(const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int r = vprintf_(fmt, ap);
    va_end(ap);
    return r;
}

int fprintf(FILE* f, const char* fmt, ...) {
    (void)f;
    va_list ap;
    va_start(ap, fmt);
    int r = vprintf_(fmt, ap);
    va_end(ap);
    return r;
}

int vfprintf(FILE* f, const char* fmt, va_list ap) {
    (void)f;
    return vprintf_(fmt, ap);
}

int sprintf(char* s, const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int r = vsnprintf_(s, (size_t)-1, fmt, ap);
    va_end(ap);
    return r;
}

int snprintf(char* s, size_t n, const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int r = vsnprintf_(s, n, fmt, ap);
    va_end(ap);
    return r;
}

int vsnprintf(char* s, size_t n, const char* fmt, va_list ap) {
    return vsnprintf_(s, n, fmt, ap);
}
