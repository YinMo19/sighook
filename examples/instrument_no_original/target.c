#include <stdio.h>

__attribute__((noinline))
int calc(int a, int b) {
    return a + b;
}

int main(void) {
    printf("calc(4, 5) = %d\n", calc(4, 5));
    return 0;
}
