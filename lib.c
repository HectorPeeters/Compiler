#include "stdio.h"
#include "stdint.h"

void printbool(uint8_t x) {
    printf("%d\n", x);
}

void print8(uint8_t x) {
    printf("%d\n", x);
}

void print16(uint16_t x) {
    printf("%d\n", x);
}

void print32(uint32_t x) {
    printf("%u\n", x);
}

void print64(uint64_t x) {
    printf("%lu\n", x);
}

void printsum(uint32_t x, uint32_t y) {
    printf("%d\n", x + y);
}