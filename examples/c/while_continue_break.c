int nonce = 1;  // For random input

int foo() {
    int sum = 0;
    int i = 0;
    int continue_num = nonce % 98;

    while (i < 100) {
        if (i == continue_num) {
            i++;
            continue;
        }
        sum += i;
        i++;

        if (i == continue_num + 2)
            break;
    }

    return sum;
}

int main() {
    return foo();
}
