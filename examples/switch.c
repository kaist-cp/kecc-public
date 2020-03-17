int main() {
    int a = 1;
    int b = 0;
    switch (a) {
        case 0: {
            b += 1;
            break;
        }
        case 1: {
            b += 2;
            break;
        }
        default: {
            b += 3;
            break;
        }
    }

    return b;
}
