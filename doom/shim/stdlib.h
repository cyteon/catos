#ifndef SHIM_STDLIB_H
#define SHIM_STDLIB_H

#include <stddef.h>

void* malloc(size_t size);
void free(void* ptr);
void* calloc(size_t nmemb, size_t size);
void* realloc(void* ptr, size_t size);

void exit(int status);

int atoi(const char* str);
double atof(const char* str);
int abs(int x);

int system(const char* command);
char* getenv(const char* name);

#endif
