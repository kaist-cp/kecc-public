int bar(int x, int y, int z) {
    int arith_mean = (x + y + z) / 3;
    int ugly_mean = (((x + y) / 2) * 2 + z) / 3;
    if (x == y) {
        return y;
    } else {
        return z;
    }
}

int main() {
    return 1;
}
