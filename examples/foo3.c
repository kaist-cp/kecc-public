int g = 10;

int foo(int, int k);

int main() {
    int i = g;
    
    return foo(i, i) == 30;
}

int foo(int i, int j) {
    return i + j + g;
}
