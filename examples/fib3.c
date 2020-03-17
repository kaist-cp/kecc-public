int fibonacci(int n) {
    int i = 0;
    int t1 = 0, t2 = 1, next_term = 0;

    if (n < 2) {
        return n;
    }

    for (i = 1; i < n; ++i) {
        next_term = t1 + t2;
        t1 = t2;
        t2 = next_term;
    }

    return t2;
}

int main() {
    return fibonacci(9);
}
