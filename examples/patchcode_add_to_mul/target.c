#include <stdio.h>

#if defined(__aarch64__)
__attribute__((naked, noinline))
int calc(int a, int b) {
    __asm__ volatile(
        "mov x8, x0\n"
        "mov x9, x1\n"
        "nop\n"
        "nop\n"
        "nop\n"
        "add w0, w8, w9\n"
        "ret\n");
}
#else
__attribute__((noinline))
int calc(int a, int b) {
    return a + b;
}
#endif

int main(void) {
    printf("calc(6, 7) = %d\n", calc(6, 7));
    return 0;
}
