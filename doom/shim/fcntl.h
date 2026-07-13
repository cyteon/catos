#ifndef SHIM_FCNTL_H
#define SHIM_FCNTL_H

#define O_RDONLY 0
#define O_WRONLY 1
#define O_RDWR 2
#define O_CREAT 0100
#define O_TRUNC 01000

int open(const char *pathname, int flags, ...);

#endif
