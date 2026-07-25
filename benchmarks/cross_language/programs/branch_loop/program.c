#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    uint64_t state = 1;
    uint64_t total = 0;
    for (uint64_t i = 1; i <= 200000; ++i) {
        state = state * 2 + i;
        if (state >= UINT64_C(1000000000)) {
            state -= UINT64_C(1000000000);
        }
        if (state >= UINT64_C(1000000000)) {
            state -= UINT64_C(1000000000);
        }
        total += state;
    }
    printf("%" PRIu64 "\n", total);
    return 0;
}
