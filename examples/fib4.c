int fibonacci(int n) {
    int i = 0;
    int t1 = 0, t2 = 1, next_term = 0;

    if (n < 2) {
        return n;
    }

    i = 1;
    while (i < n) {
        next_term = t1 + t2;
        t1 = t2;
        t2 = next_term;
        ++i;
    }

    return t2;
}

int main() {
    return fibonacci(9) == 34;
}
