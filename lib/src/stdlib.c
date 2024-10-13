#include <sio.h>
#include <slib.h>

__attribute__((__noreturn__)) void abort(void)
{
    printf("kernel: panic: abort()\n");
    while (1) {
    }
    __builtin_unreachable();
}
