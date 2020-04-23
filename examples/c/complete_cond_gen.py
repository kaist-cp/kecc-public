#!/usr/bin/env python3

"""Make c program which uses complicated conditional expression

To make c program, execute `python3 make_cond.py > file_name.c`
"""

import random

def eval_cond(arr_cond):
    """Evaluate conditional expression
    """
    if len(arr_cond) == 1:
        return arr_cond[0]
    new_arr_cond = []
    for cond_start in range(len(arr_cond) // 3):
        cond_val = arr_cond[3*cond_start + 1] if arr_cond[3*cond_start] else arr_cond[3*cond_start + 2]
        new_arr_cond.append(cond_val)
    return eval_cond(new_arr_cond)

def make_func(i):
    """Make a function that contains a conditional expression
    """
    func_signature = "int " + "func_" + str(i) + "()"
    variables = "abcdefghijklmnopqrstuvwxyzA"
    func_inner = []
    val_bitmap = []

    # Variable initializiation
    for var in variables:
        val = random.randint(0, 1)
        val_bitmap.append(val)
        decl = "\tint " + var + " = " + str(val) + ";"
        func_inner.append(decl)

    expr_val = eval_cond(val_bitmap)
    func_inner.append("\treturn (((a ? b : c) ? (d ? e : f) : (g ? h : i)) ? ((j ? k : l) ? (m ? n : o) : (p ? q : r)) : ((s ? t : u) ? (v ? w : x) : (y ? z : A))) == " + str(expr_val) + ";")

    return "\n".join([func_signature, "{"] + func_inner + ["}"])

if __name__ == "__main__":
    src = ""
    return_stmt = "\treturn ("
    NUM_FUNC = 100
    for i in range(NUM_FUNC):
        src += make_func(i)
        src += "\n\n"
        return_stmt += "func_" + str(i) + "()"
        return_stmt += " && " if i != (NUM_FUNC - 1) else ") == "
    return_stmt += "1;"
    src += "int main()\n{\n" + return_stmt + "\n}\n"

    print(src)
