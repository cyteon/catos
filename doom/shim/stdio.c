#include <stdarg.h>
#include <stddef.h>

extern void _putchar(int c);

static void out(char **buf, size_t *n, int c)
{
    if (buf) {
        if (*n)
            **buf = c, (*buf)++;
    } else {
        _putchar(c);
    }

    (*n)++;
}


static void str(char **buf, size_t *n, const char *s)
{
    while (*s)
        out(buf, n, *s++);
}

static void num(char **buf, size_t *n, int x)
{
    char b[16];
    int i = 0;

    if (x < 0) {
        out(buf, n, '-');
        x = -x;
    }

    do {
        b[i++] = '0' + x % 10;
        x /= 10;
    } while (x);

    while (i)
        out(buf, n, b[--i]);
}

static int vfmt(char *buf, size_t size, const char *fmt, va_list ap)
{
    char *p = buf;
    size_t n = 0;

    while (*fmt) {
        if (*fmt != '%') {
            out(buf ? &p : 0, &n, *fmt++);
            continue;
        }

        switch (*++fmt) {
        case 's': str(buf ? &p : 0, &n, va_arg(ap, char *)); break;
        case 'd': num(buf ? &p : 0, &n, va_arg(ap, int)); break;
        case 'c': out(buf ? &p : 0, &n, va_arg(ap, int)); break;
        case '%': out(buf ? &p : 0, &n, '%'); break;
        }

        fmt++;
    }

    if (buf && size)
        *p = 0;

    return n;
}

int printf(const char *f, ...)
{
    va_list ap;
    va_start(ap, f);
    int r = vfmt(0, 0, f, ap);
    va_end(ap);
    return r;
}

int fprintf(void *x, const char *f, ...)
{
    (void)x;
    va_list ap;
    va_start(ap, f);
    int r = vfmt(0, 0, f, ap);
    va_end(ap);
    return r;
}

int vfprintf(void *x, const char *f, va_list ap)
{
    (void)x;
    return vfmt(0, 0, f, ap);
}

int snprintf(char *b, size_t s, const char *f, ...)
{
    va_list ap;
    va_start(ap, f);
    int r = vfmt(b, s, f, ap);
    va_end(ap);
    return r;
}

int vsnprintf(char *b, size_t s, const char *f, va_list ap)
{
    return vfmt(b, s, f, ap);
}
