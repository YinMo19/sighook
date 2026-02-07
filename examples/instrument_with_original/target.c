#include <stdio.h>

#if defined(__linux__) && defined(__aarch64__)
int calc(int a, int b);

__asm__(
    ".text\n"
    ".global calc\n"
    ".global calc_add_insn\n"
    ".type calc, %function\n"
    "calc:\n"
    "  mov x8, x0\n"
    "  mov x9, x1\n"
    "  nop\n"
    "  nop\n"
    "  nop\n"
    "calc_add_insn:\n"
    "  add w0, w8, w9\n"
    "  ret\n"
    ".size calc, .-calc\n");
#elif defined(__linux__) && defined(__x86_64__)
__asm__(".global calc_add_insn\n");

__attribute__((noinline))
int calc(int a, int b) {
    int sum = a + b;
    __asm__ volatile(
        "calc_add_insn:\n"
        "  nop\n");
    return sum;
}
#elif defined(__aarch64__)
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
    printf("calc(1, 2) = %d\n", calc(1, 2));
    return 0;
}
