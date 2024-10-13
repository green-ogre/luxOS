#ifndef STDIO_H_
#define STDIO_H_

#include <stdbool.h>
#include <stddef.h>

#define EOF (-1)

int putchar(int ic);
int printf(const char *__restrict format, ...);
int puts(const char *string);

#endif
