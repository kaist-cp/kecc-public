use std::collections::VecDeque;

use itertools::izip;

use crate::ir::*;
use crate::utils::IsEquiv;
use crate::*;

impl IsEquiv for TranslationUnit {
    fn is_equiv(&self, other: &Self) -> bool {
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

    if lhs.instructions != rhs.instructions {
        return false;
    }

    is_equiv_block_exit(&lhs.exit, &rhs.exit, map)
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
