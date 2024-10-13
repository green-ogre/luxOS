#ifndef STDIO_H_
#define STDIO_H_

#include <stddef.h>

int memcmp(const void *aptr, const void *bptr, size_t size);
void *memcpy(void *__restrict dstptr, const void *__restrict srcptr,
             size_t size);
void *memmove(void *dstptr, const void *srcptr, size_t size);
void *memset(void *bufptr, int value, size_t size);
size_t strlen(const char *str);

#endif
