#ifndef SHIM_UNISTD_H
#define SHIM_UNISTD_H

int access(const char *pathname, int mode);
#define R_OK 4
#define W_OK 2
#define X_OK 1
#define F_OK 0

#endif
