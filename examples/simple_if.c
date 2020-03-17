int (fibonacci)(int n) {
    if (n < 2) {
        n += 2;
    }

    return fibonacci(n - 2) + fibonacci(n - 1);
}
