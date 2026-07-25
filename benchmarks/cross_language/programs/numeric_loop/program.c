#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    uint64_t state = 1;
    uint64_t total = 0;
    for (uint64_t i = 1; i <= 200000; ++i) {
        state = (state * UINT64_C(1664525) + UINT64_C(1013904223)) %
                UINT64_C(4294967296);
        total = (total + state) % UINT64_C(9007199254740881);
    }
    printf("%" PRIu64 "\n", total);
    return 0;
}
