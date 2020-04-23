int foo() {
    int sum = 0;
    int i = 0;

    while(i < 10) {
        if(i == 3) {
            i++;
            continue;
        }
        sum += i;
        i++;

        if(i == 5) break;
    }

    return sum;
}

int main() {
    return foo() == 7;
}
