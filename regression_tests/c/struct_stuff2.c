typedef struct {
  struct {
    int a[4];
  };
} b;
int main() {
  b c = {};
  return c.a[2];
}
