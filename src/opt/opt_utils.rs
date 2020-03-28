#![allow(dead_code)]

use std::collections::HashMap;

use crate::ir::*;

/// "Replace-all-uses-with".
pub trait Walk {
    fn walk<F>(&mut self, f: F) -> bool
    where
        F: FnMut(&mut Operand) -> bool;
}

impl Walk for FunctionDefinition {
    fn walk<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Operand) -> bool,
    {
        self.blocks.iter_mut().any(|(_, block)| block.walk(&mut f))
    }
}

impl Walk for Block {
    fn walk<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Operand) -> bool,
    {
        self.instructions.iter_mut().any(|i| i.walk(&mut f)) || self.exit.walk(&mut f)
    }
}

impl Walk for Instruction {
    fn walk<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Operand) -> bool,
    {
        match self {
            Self::Nop => false,
            Self::BinOp { lhs, rhs, .. } => lhs.walk(&mut f) || rhs.walk(&mut f),
            Self::UnaryOp { operand, .. } => operand.walk(&mut f),
            Self::Store { ptr, value } => ptr.walk(&mut f) || value.walk(&mut f),
            Self::Load { ptr } => ptr.walk(&mut f),
            Self::Call { callee, args, .. } => {
                callee.walk(&mut f) || args.iter_mut().any(|a| a.walk(&mut f))
            }
            Self::TypeCast { value, .. } => value.walk(&mut f),
        }
    }
}

impl Walk for BlockExit {
    fn walk<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Operand) -> bool,
    {
        match self {
            Self::Jump { arg } => arg.walk(&mut f),
            Self::ConditionalJump {
                condition,
                arg_then,
                arg_else,
            } => condition.walk(&mut f) || arg_then.walk(&mut f) || arg_else.walk(&mut f),
            Self::Switch {
                value,
                default,
                cases,
            } => {
                value.walk(&mut f)
                    || default.walk(&mut f)
                    || cases.iter_mut().any(|(_, a)| a.walk(&mut f))
            }
            Self::Return { value } => value.walk(&mut f),
            Self::Unreachable => false,
        }
    }
}

impl Walk for JumpArg {
    fn walk<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Operand) -> bool,
    {
        self.args.iter_mut().any(|a| a.walk(&mut f))
    }
}

impl Walk for Operand {
    fn walk<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Operand) -> bool,
    {
        f(self)
    }
}

pub fn replace_operand(operand: &mut Operand, from: &Operand, to: &Operand) -> bool {
    if operand == from {
        *operand = to.clone();
        true
    } else {
        false
    }
}

pub fn replace_operands(operand: &mut Operand, dict: &HashMap<RegisterId, Operand>) -> bool {
    if let Some((rid, dtype)) = operand.get_register_mut() {
        if let Some(val) = dict.get(rid) {
            assert_eq!(*dtype, val.dtype());
            *operand = val.clone();
            return true;
        }
    }
    false
}

pub fn make_cfg(fdef: &FunctionDefinition) -> HashMap<BlockId, Vec<JumpArg>> {
    let mut result = HashMap::new();

    for (bid, block) in &fdef.blocks {
        let mut args = Vec::new();
        match &block.exit {
            BlockExit::Jump { arg } => args.push(arg.clone()),
            BlockExit::ConditionalJump {
                arg_then, arg_else, ..
            } => {
                args.push(arg_then.clone());
                args.push(arg_else.clone());
            }
            BlockExit::Switch { default, cases, .. } => {
                args.push(default.clone());
                for (_, arg) in cases {
                    args.push(arg.clone());
                }
            }
            _ => {}
        }
        result.insert(*bid, args);
    }
    result
}

pub fn reverse_cfg(
    cfg: &HashMap<BlockId, Vec<JumpArg>>,
) -> HashMap<BlockId, Vec<(BlockId, JumpArg)>> {
    let mut result = HashMap::new();

    for (bid, jumps) in cfg {
        for jump in jumps {
            result
                .entry(jump.bid)
                .or_insert_with(Vec::new)
                .push((*bid, jump.clone()));
        }
    }
    result
}

pub struct Domtree {}

impl Domtree {
    pub fn walk<F>(&self, _f: F)
    where
        F: FnMut(BlockId, BlockId),
    {
        todo!()
    }
}

pub fn make_domtree(_cfg: &HashMap<BlockId, Vec<JumpArg>>) -> Domtree {
    todo!()
}
