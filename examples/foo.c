int foo(int x, int y, int z){
    if (x == y) { return y; }
    else { return z; }
}

int main() {
    return foo(0, 1, -1) == -1;
}
