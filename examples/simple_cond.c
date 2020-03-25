int f(int x) {
    return x + 8;
}

int main() {
    int x = 0;
    int y = (x++ == 1) ? 1 : 2;

    return f((x < y) ? x : 2) == 9;
}
