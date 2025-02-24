int main() {
    int a[10];
    int* p = a;

    for (int i = 0; i < 10; i++) {
        *(p++) = i;
    }

    return a[5] == 5;
}
