typedef struct {
    char a;
} MyStruct;

int main() {
    const MyStruct mystruct = { 1 };

    MyStruct temp2;
    temp2 = mystruct;

    return temp2.a;
}
