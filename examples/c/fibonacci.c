int nonce = 1; // For random input

int fibonacci(int n) {
    if (n < 2) {
        return n;
    }

    return fibonacci(n - 2) + fibonacci(n - 1);
}

int main() {
    int number = nonce % 20;
    return fibonacci(number);
}
