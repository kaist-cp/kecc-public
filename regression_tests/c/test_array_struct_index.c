typedef struct {
  struct {
    int a[4][5];
  };
} b;
b c;
int main() {
  for (int d = 0; d < 4; d++)
    for (int e = 0; e < 5; e++)
      c.a[d][e];
}
