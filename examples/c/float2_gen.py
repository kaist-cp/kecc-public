#!/usr/bin/env python3

"""Make c program which uses floating point variables

To make c program, execute `python3 float_gen.py > file_name.c`
"""

import random

def random_operator():
    ops = ["+", "-", "*", "/"]
    return ops[random.randint(0, len(ops) - 1)]

def random_dtype():
    dtypes = ["float", "double"]
    return dtypes[random.randint(0, len(dtypes) - 1)]

def random_suffix():
    suffixes = ["f", ""]
    return suffixes[random.randint(0, len(suffixes) - 1)]

def make_expr(vars):
    if len(vars) == 1:
        return vars[0]
    else:
        var = vars.pop()
        return "(" + var + " " + random_operator() + " " + make_expr(vars) + ")"

def make_func(i):
    """Make a function that contains a conditional expression
    """
    func_signature = random_dtype() + " func_" + str(i) + "()"
    variables = "abcdefghijklmnopqrstuvwxyzA"
    func_inner = []
    val_bitmap = []

    # Variable initializiation
    for var in variables:
        val = random.gauss(0, 1)
        val_bitmap.append(val)
        decl = "\t" + random_dtype() + " " + var + " = " + str(val) + random_suffix() + ";"
        func_inner.append(decl)

    func_inner.append("\treturn " + make_expr(list(variables)) + ";")

    return "\n".join([func_signature, "{"] + func_inner + ["}"])

if __name__ == "__main__":
    src = ""
    return_stmt = "\treturn (int)("
    NUM_FUNC = 100
    for i in range(NUM_FUNC):
        src += make_func(i)
        src += "\n\n"
        return_stmt += "func_" + str(i) + "()"
        return_stmt += " " + random_operator() + " " if i != (NUM_FUNC - 1) else ");"
    src += "int main()\n{\n" + return_stmt + "\n}\n"

    print(src)
