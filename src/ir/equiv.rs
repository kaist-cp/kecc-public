use std::collections::VecDeque;

use itertools::izip;

use crate::ir::*;
use crate::utils::IsEquiv;
use crate::*;

impl IsEquiv for TranslationUnit {
    fn is_equiv(&self, other: &Self) -> bool {
        if self.decls.len() != other.decls.len() {
            return false;
        }

        for (lhs, rhs) in izip!(&self.decls, &other.decls) {
            if lhs.0 != rhs.0 {
                return false;
            }

            if !lhs.1.is_equiv(rhs.1) {
                return false;
            }
        }

        if self.structs != other.structs {
            return false;
        }

        true
    }
}

impl IsEquiv for ir::Declaration {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Variable { dtype, initializer },
                Self::Variable {
                    dtype: dtype_other,
                    initializer: initializer_other,
                },
            ) => {
                if dtype != dtype_other {
                    return false;
                }

                initializer.is_equiv(initializer_other)
            }
            (
                Self::Function {
                    signature,
                    definition,
                },
                Self::Function {
                    signature: signature_other,
                    definition: definition_other,
                },
            ) => {
                if signature != signature_other {
                    return false;
                }

                definition.is_equiv(definition_other)
            }
            _ => false,
        }
    }
}

fn traverse_preorder(blocks: &BTreeMap<BlockId, Block>, bid: BlockId) -> Vec<BlockId> {
    let mut result = vec![bid];
    let mut queue = VecDeque::new();
    queue.push_back(bid);

    loop {
        while let Some(bid) = queue.pop_front() {
            let next = match &blocks.get(&bid).unwrap().exit {
                BlockExit::Jump { arg } => vec![arg.bid],
                BlockExit::ConditionalJump {
                    arg_then, arg_else, ..
                } => vec![arg_then.bid, arg_else.bid],
                BlockExit::Switch { default, cases, .. } => {
                    let mut next = cases.iter().map(|(_, a)| a.bid).collect::<Vec<_>>();
                    next.push(default.bid);
                    next
                }
                _ => Vec::new(),
            };
            for n in next {
                if !result.contains(&n) {
                    result.push(n);
                    queue.push_back(n);
                }
            }
        }

        if let Some(bid) = blocks.keys().find(|b| !result.contains(b)) {
            result.push(*bid);
            queue.push_back(*bid);
        } else {
            break;
        }
    }

    result
}

fn is_equiv_block(lhs: &Block, rhs: &Block, map: &HashMap<BlockId, BlockId>) -> bool {
    if lhs.phinodes != rhs.phinodes {
        return false;
    }

    if lhs.instructions.len() != rhs.instructions.len() {
        return false;
    }

    for (l, r) in izip!(&lhs.instructions, &rhs.instructions) {
        if !is_equiv_instruction(l, r, map) {
            return false;
        }
    }

    is_equiv_block_exit(&lhs.exit, &rhs.exit, map)
}

fn is_equiv_instruction(
    lhs: &Instruction,
    rhs: &Instruction,
    map: &HashMap<BlockId, BlockId>,
) -> bool {
    match (lhs, rhs) {
        (Instruction::Nop, Instruction::Nop) => true,
        (
            Instruction::BinOp {
                op,
                lhs,
                rhs,
                dtype,
            },
            Instruction::BinOp {
                op: op_other,
                lhs: lhs_other,
                rhs: rhs_other,
                dtype: dtype_other,
            },
        ) => {
            op == op_other
                && is_equiv_operand(lhs, lhs_other, map)
                && is_equiv_operand(rhs, rhs_other, map)
                && dtype == dtype_other
        }
        (
            Instruction::UnaryOp { op, operand, dtype },
            Instruction::UnaryOp {
                op: op_other,
                operand: operand_other,
                dtype: dtype_other,
            },
        ) => {
            op == op_other && is_equiv_operand(operand, operand_other, map) && dtype == dtype_other
        }
        (
            Instruction::Store { ptr, value },
            Instruction::Store {
                ptr: ptr_other,
                value: value_other,
            },
        ) => is_equiv_operand(ptr, ptr_other, map) && is_equiv_operand(value, value_other, map),
        (Instruction::Load { ptr }, Instruction::Load { ptr: ptr_other }) => {
            is_equiv_operand(ptr, ptr_other, map)
        }
        (
            Instruction::Call {
                callee,
                args,
                return_type,
            },
            Instruction::Call {
                callee: callee_other,
                args: args_other,
                return_type: return_type_other,
            },
        ) => {
            is_equiv_operand(callee, callee_other, map)
                && args.len() == args_other.len()
                && izip!(args, args_other).all(|(l, r)| is_equiv_operand(l, r, map))
                && return_type == return_type_other
        }
        (
            Instruction::TypeCast {
                value,
                target_dtype,
            },
            Instruction::TypeCast {
                value: value_other,
                target_dtype: target_dtype_other,
            },
        ) => is_equiv_operand(value, value_other, map) && target_dtype == target_dtype_other,
        (
            Instruction::GetElementPtr { ptr, offset, dtype },
            Instruction::GetElementPtr {
                ptr: ptr_other,
                offset: offset_other,
                dtype: dtype_other,
            },
        ) => {
            is_equiv_operand(ptr, ptr_other, map)
                && is_equiv_operand(offset, offset_other, map)
                && dtype == dtype_other
        }
        _ => false,
    }
}

fn is_equiv_operand(lhs: &Operand, rhs: &Operand, map: &HashMap<BlockId, BlockId>) -> bool {
    match (lhs, rhs) {
        (Operand::Constant(_), Operand::Constant(_)) => lhs == rhs,
        (
            Operand::Register { rid, dtype },
            Operand::Register {
                rid: rid_other,
                dtype: dtype_other,
            },
        ) => is_equiv_rid(rid, rid_other, map) && dtype == dtype_other,
        _ => false,
    }
}

fn is_equiv_rid(lhs: &RegisterId, rhs: &RegisterId, map: &HashMap<BlockId, BlockId>) -> bool {
    match (lhs, rhs) {
        (RegisterId::Local { .. }, RegisterId::Local { .. }) => lhs == rhs,
        (
            RegisterId::Arg { bid, aid },
            RegisterId::Arg {
                bid: bid_other,
                aid: aid_other,
            },
        ) => map.get(bid) == Some(bid_other) && aid == aid_other,
        (
            RegisterId::Temp { bid, iid },
            RegisterId::Temp {
                bid: bid_other,
                iid: iid_other,
            },
        ) => map.get(bid) == Some(bid_other) && iid == iid_other,
        _ => false,
    }
}

fn is_equiv_block_exit(lhs: &BlockExit, rhs: &BlockExit, map: &HashMap<BlockId, BlockId>) -> bool {
    match (lhs, rhs) {
        (BlockExit::Jump { arg }, BlockExit::Jump { arg: arg_other }) => {
            is_equiv_arg(arg, arg_other, map)
        }
        (
            BlockExit::ConditionalJump {
                condition,
                arg_then,
                arg_else,
            },
            BlockExit::ConditionalJump {
                condition: condition_other,
                arg_then: arg_then_other,
                arg_else: arg_else_other,
            },
        ) => {
            if condition != condition_other {
                return false;
            }
            if !is_equiv_arg(arg_then, arg_then_other, map) {
                return false;
            }
            if !is_equiv_arg(arg_else, arg_else_other, map) {
                return false;
            }
            true
        }
        (
            BlockExit::Switch {
                value,
                default,
                cases,
            },
            BlockExit::Switch {
                value: value_other,
                default: default_other,
                cases: cases_other,
            },
        ) => {
            if value != value_other {
                return false;
            }
            if !is_equiv_arg(default.deref(), default_other.deref(), map) {
                return false;
            }
            if cases.len() != cases_other.len() {
                return false;
            }
            for (l, r) in izip!(cases, cases_other) {
                if l.0 != r.0 {
                    return false;
                }
                if !is_equiv_arg(&l.1, &r.1, map) {
                    return false;
                }
            }
            true
        }
        _ => lhs == rhs,
    }
}

fn is_equiv_arg(lhs: &JumpArg, rhs: &JumpArg, map: &HashMap<BlockId, BlockId>) -> bool {
    if map.get(&lhs.bid) != Some(&rhs.bid) {
        return false;
    }
    if lhs.args != rhs.args {
        return false;
    }
    true
}

impl IsEquiv for ir::FunctionDefinition {
    fn is_equiv(&self, other: &Self) -> bool {
        if self.allocations != other.allocations {
            return false;
        }

        if self.blocks.len() != other.blocks.len() {
            return false;
        }

        if self.bid_init != other.bid_init {
            return false;
        }

        let preorder = traverse_preorder(&self.blocks, self.bid_init);
        let preorder_other = traverse_preorder(&other.blocks, other.bid_init);
        assert_eq!(preorder.len(), preorder_other.len());

        let mut map = HashMap::new();
        for (f, t) in izip!(&preorder, &preorder_other) {
            map.insert(*f, *t);
        }

        if map.get(&self.bid_init) != Some(&other.bid_init) {
            return false;
        }

        for (f, t) in &map {
            let lhs = self.blocks.get(f).unwrap();
            let rhs = other.blocks.get(t).unwrap();
            if !is_equiv_block(lhs, rhs, &map) {
                return false;
            }
        }

        true
    }
}
