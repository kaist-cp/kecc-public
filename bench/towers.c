//**************************************************************************
// Towers of Hanoi benchmark
//--------------------------------------------------------------------------
//
// Towers of Hanoi is a classic puzzle problem. The game consists of
// three pegs and a set of discs. Each disc is a different size, and
// initially all of the discs are on the left most peg with the smallest
// disc on top and the largest disc on the bottom. The goal is to move all
// of the discs onto the right most peg. The catch is that you are only
// allowed to move one disc at a time and you can never place a larger
// disc on top of a smaller disc.
//
// This implementation starts with NUM_DISC discs and uses a recursive
// algorithm to solve the puzzle.

struct Node {
    int val;
    struct Node* next;
};

struct List {
    int size;
    struct Node* head;
};

struct List g_nodeFreeList;
struct Node g_nodePool[7];

int list_getSize(struct List* list) {
    return list->size;
}

void list_init(struct List* list) {
    list->size = 0;
    list->head = 0;
}

void list_push(struct List* list, int val) {
    struct Node* newNode;

    // Pop the next free node off the free list
    newNode = g_nodeFreeList.head;
    g_nodeFreeList.head = g_nodeFreeList.head->next;

    // Push the new node onto the given list
    newNode->next = list->head;
    list->head = newNode;

    // Assign the value
    list->head->val = val;

    // Increment size
    list->size++;
}

int list_pop(struct List* list) {
    struct Node* freedNode;
    int val;

    // Get the value from the->head of given list
    val = list->head->val;

    // Pop the head node off the given list
    freedNode = list->head;
    list->head = list->head->next;

    // Push the freed node onto the free list
    freedNode->next = g_nodeFreeList.head;
    g_nodeFreeList.head = freedNode;

    // Decrement size
    list->size--;

    return val;
}

void list_clear(struct List* list) {
    while (list_getSize(list) > 0)
        list_pop(list);
}

//--------------------------------------------------------------------------
// Tower data structure and functions

struct Towers {
    int numDiscs;
    int numMoves;
    struct List pegA;
    struct List pegB;
    struct List pegC;
};

void towers_init(struct Towers* towers, int n, int nonce) {
    int i;

    towers->numDiscs = n;
    towers->numMoves = 0;

    list_init(&(towers->pegA));
    list_init(&(towers->pegB));
    list_init(&(towers->pegC));

    for (i = 0; i < n; i++)
        list_push(&(towers->pegA), nonce * (n - i));
}

void towers_clear(struct Towers* towers, int nonce) {
    list_clear(&(towers->pegA));
    list_clear(&(towers->pegB));
    list_clear(&(towers->pegC));

    towers_init(towers, towers->numDiscs, nonce);
}

void towers_solve_h(struct Towers* towers, int n, struct List* startPeg, struct List* tempPeg, struct List* destPeg) {
    int val;

    if (n == 1) {
        val = list_pop(startPeg);
        list_push(destPeg, val);
        towers->numMoves++;
    } else {
        towers_solve_h(towers, n - 1, startPeg, destPeg, tempPeg);
        towers_solve_h(towers, 1, startPeg, tempPeg, destPeg);
        towers_solve_h(towers, n - 1, tempPeg, startPeg, destPeg);
    }
}

void towers_solve(struct Towers* towers) {
    towers_solve_h(towers, towers->numDiscs, &(towers->pegA), &(towers->pegB), &(towers->pegC));
}

int towers_verify(struct Towers* towers, int nonce) {
    struct Node* ptr;
    int numDiscs = 0;
    int result = 0;

    if (list_getSize(&towers->pegA) != 0) {
        return 2;
    }

    if (list_getSize(&towers->pegB) != 0) {
        return 3;
    }

    if (list_getSize(&towers->pegC) != towers->numDiscs) {
        return 4;
    }

    for (ptr = towers->pegC.head; ptr != 0; ptr = ptr->next) {
        numDiscs++;
        if (ptr->val != nonce * numDiscs) {
            return 5;
        }
        result += ptr->val;
    }

    if (towers->numMoves != ((1 << towers->numDiscs) - 1)) {
        return 6;
    }

    return 0;
}

//--------------------------------------------------------------------------
// Main

int run_towers(int dummy_0, int nonce) {
    struct Towers towers;
    int i;

    // Initialize free list

    list_init(&g_nodeFreeList);
    g_nodeFreeList.head = &(g_nodePool[0]);
    g_nodeFreeList.size = 7;
    g_nodePool[7 - 1].next = 0;
    g_nodePool[7 - 1].val = 99;
    for (i = 0; i < (7 - 1); i++) {
        g_nodePool[i].next = &(g_nodePool[i + 1]);
        g_nodePool[i].val = nonce * i;
    }

    towers_init(&towers, 7, nonce);

    // Solve it

    towers_clear(&towers, nonce);
    towers_solve(&towers);

    // Check the results
    return towers_verify(&towers, nonce);
}
