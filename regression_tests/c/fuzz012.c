int a;
void b() {
  int *c = &a;
  c = c;
}
int main() {}
