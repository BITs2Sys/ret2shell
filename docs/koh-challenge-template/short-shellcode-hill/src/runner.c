#define _GNU_SOURCE
#include <ctype.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static int hexval(int c) {
  if ('0' <= c && c <= '9') {
    return c - '0';
  }
  if ('a' <= c && c <= 'f') {
    return c - 'a' + 10;
  }
  if ('A' <= c && c <= 'F') {
    return c - 'A' + 10;
  }
  return -1;
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "usage: %s HEXSHELLCODE\n", argv[0]);
    return 2;
  }

  const char *hex = argv[1];
  size_t hex_len = strlen(hex);
  if (hex_len == 0 || (hex_len % 2) != 0 || hex_len > 1024) {
    fprintf(stderr, "invalid shellcode length\n");
    return 2;
  }

  size_t code_len = hex_len / 2;
  uint8_t *code = mmap(NULL, 4096, PROT_READ | PROT_WRITE | PROT_EXEC, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
  if (code == MAP_FAILED) {
    perror("mmap");
    return 1;
  }

  for (size_t i = 0; i < code_len; i++) {
    int hi = hexval((unsigned char)hex[i * 2]);
    int lo = hexval((unsigned char)hex[i * 2 + 1]);
    if (hi < 0 || lo < 0) {
      fprintf(stderr, "non-hex input\n");
      return 2;
    }
    code[i] = (uint8_t)((hi << 4) | lo);
  }

  alarm(1);
  ((void (*)(void))code)();
  return 0;
}
