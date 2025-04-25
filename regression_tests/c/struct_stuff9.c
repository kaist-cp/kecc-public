struct a {
  long b;
} c;
struct a d(void) {
  return c;
}
int main() {
  struct a e = d();
  return e.b;
}
