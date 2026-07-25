#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>

static uint64_t mix(uint64_t state, uint64_t index) {
    return (state * UINT64_C(48271) + index) % UINT64_C(2147483647);
}

int main(void) {
    uint64_t state = 1;
    for (uint64_t i = 1; i <= 100000; ++i) {
        state = mix(state, i);
    }
    printf("%" PRIu64 "\n", state);
    return 0;
}
