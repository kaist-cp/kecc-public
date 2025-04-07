int g = 0;

int* foo() {
    g += 10;
    return &g;
}

int main() {
    // `foo()` should be called once.
    *&*foo() += 1;

    return g;
}
