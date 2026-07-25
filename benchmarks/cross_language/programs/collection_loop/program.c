#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    const size_t length = 50000;
    uint64_t *values = malloc(length * sizeof(*values));
    if (values == NULL) {
        return 2;
    }
    for (size_t i = 0; i < length; ++i) {
        values[i] = i % 997;
    }
    uint64_t total = 0;
    for (size_t i = 0; i < length; ++i) {
        total += values[i];
    }
    free(values);
    printf("%" PRIu64 "\n", total);
    return 0;
}
