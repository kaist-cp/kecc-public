int main() {
  int i = 0;
  int c = 0;
  while (i < 10) {
    i++;
    switch (i) {
      case (1): {
        continue;
        break;
      }
      default: {
        break;
      }
    }
    c++;
  }
  return c;
}
