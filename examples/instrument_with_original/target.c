#include <stdio.h>

__attribute__((noinline))
int calc(int a, int b) {
    return a + b;
}

int main(void) {
    printf("calc(1, 2) = %d\n", calc(1, 2));
    return 0;
}
