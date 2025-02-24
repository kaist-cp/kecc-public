int main() {
  int a = 1;
  int b = 2;
  do {
    int t = a;
    a = b;
    b = t;
  } while (b == 1);
  return a * 10 + b;
}
