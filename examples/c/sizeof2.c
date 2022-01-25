int main() {
    char a = 42, b = 5;
    long c[10];

    return sizeof(a) == 1 && sizeof(a + b) == 4 && sizeof(c) == 80;
}
