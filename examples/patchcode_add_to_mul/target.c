#include <stdio.h>

__attribute__((noinline))
int calc(int a, int b) {
    return a + b;
}

int main(void) {
    printf("calc(6, 7) = %d\n", calc(6, 7));
    return 0;
}
