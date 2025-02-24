int sum(int len, int* p) {
    int result = 0;
    for (int i = 0; i < len; i++) {
        result += p[i];
    }

    return result;
}

int main() {
    int a[5];
    int len = 5;

    for (int i = 0; i < len; i++) {
        a[i] = i;
    }

    return sum(len, a) == 10;
}
