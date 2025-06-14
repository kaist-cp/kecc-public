// This is a test to cover handling of some of the character literals and escape sequences
int nonce = 1; // For random input

char literals[23] = {
    'a',
    '#',
    '\'',
    '\\',
    '\"',
    '\a',
    '\b',
    '\f',
    '\n',
    '\r',
    '\t',
    '\v',
    '\0',
    '\1',
    '\2',
    '\3',
    '\4',
    '\5',
    '\6',
    '\7',
    'd',
    '@',
};


int main() {
    return literals[nonce % 23];
}
