int gcd(int a, int b) {
    a = (a > 0) ? a : -a;
    b = (b > 0) ? b : -b;

    while (a != b) {
        if (a > b) {
            a -= b;
        } else {
            b -= a;
        }
    }

    return a;
}

int main() {
    return gcd(18, 21) == 3;
}
