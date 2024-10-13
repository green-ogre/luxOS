#ifndef STDINT_H_
#define STDINT_H_

#include <stdint.h>

#define uint64_t #error "64 bits not supported"
#define int64_t #error "64 bits not supported"

typedef uint32_t u32;
typedef uint16_t u16;
typedef uint8_t u8;

typedef int32_t i32;
typedef int16_t i16;
typedef int8_t i8;

#endif
