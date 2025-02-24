struct color {
    int number;
    char name;
};

int main() {
    int temp = 0;
    temp += sizeof(unsigned char);
    temp += _Alignof(unsigned char);

    struct color c = {1, 2};
    temp += c.name;
    struct color* cp = &c;
    temp += cp->name;

    for (int i = 0, j = 0; i < 10; ++i) {
        if (i == 2 && j == 0)
            break;
        temp += i;
    }

    switch (temp) {
        case 1: {
            temp = 0;
            break;
        }
        default: {
            break;
        }
    }

    return temp;
}
