int* foo(int a[10]) {
    return a;
}

int main() {
    int a[10];

    for (int i = 0; i < 10; i++) {
        (foo(a))[i] = i;
    }

    return a[5] == 5;
}
