int foo() {
    int sum = 0;

    for (int i = 0;;) {
        if (i == 5)
            break;
        if (i == 3) {
            i++;
            continue;
        }
        sum += i;
        i++;
    }

    return sum;
}

int main() {
    return foo() == 7;
}
