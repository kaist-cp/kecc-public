#include <stdio.h>

unsigned long read_cycles()
{
    unsigned long cycles;
    asm volatile ("rdcycle %0" : "=r" (cycles));
    return cycles;
}

extern int job();

int main() {
  unsigned long start, end;
  int answer;

  start = read_cycles();
  answer = job();
  end = read_cycles();

  printf("cycles: %lu\n", end - start);
  printf("answer: %d\n", answer);

  return 0;
}
