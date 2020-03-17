int fibonacci(int n) {
    if (n < 2) {
        return n;
    }

    return fibonacci(n - 2) + fibonacci(n - 1);
}

int main() {
    return fibonacci(9);
}
