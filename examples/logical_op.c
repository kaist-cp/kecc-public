int main() {
    int a = 0;
    int b = 0;
    int c = 0;
    int d = 0;

    if ((a = 1) || (b = 1)) {
        b++;
    }

    if ((c = 1) && (d = 1)) {
        d++;
    }

    return b == 1 && d == 2;
}
