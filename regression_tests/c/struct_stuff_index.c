typedef struct {
  struct {
    int a[4];
  };
} b;
b c;
int main() { return c.a[2]; }
