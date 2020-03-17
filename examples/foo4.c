int foo(int i, int j, int k) {
  return i + j + k;
}

int (* foo2())(int, int, int){
  return foo;
}

int (* (* foo3())())(int, int, int){
  return foo2;
}

int main() {
  return foo3()()(2, 2, 2) == 6;
}
