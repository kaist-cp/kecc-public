//! Visualize IR.

use std::collections::HashMap;

use crate::ir::*;
use crate::Translate;

#[derive(Default, Debug)]
pub struct Visualizer {
    /// First instruction in the function.
    function_first_instruction: HashMap<String, String>,

    /// First instruction in the block.
    block_first_instruction: HashMap<(String, BlockId), String>,
}

impl Translate<TranslationUnit> for Visualizer {
    type Target = String;
    type Error = ();

    fn translate(&mut self, source: &TranslationUnit) -> Result<Self::Target, Self::Error> {
        let mut subgraphs = Vec::new();

        // TODO: Add variables and structs information
        for (name, decl) in &source.decls {
            match decl {
                Declaration::Variable { .. } => {}
                Declaration::Function {
                    signature,
                    definition,
                } => {
                    let Some(definition) = definition else {
                        continue;
                    };
                    let subgraph = self.translate_function(name, signature, definition)?;
                    subgraphs.push(subgraph);
                }
            }
        }

        let mut edges = Vec::new();

        // Add edges between subgraphs
        for (name, decl) in &source.decls {
            if let Declaration::Function { definition, .. } = decl {
                let Some(definition) = definition else {
                    continue;
                };

                for (bid, block) in &definition.blocks {
                    for (iid, instruction) in block.instructions.iter().enumerate() {
                        if let Instruction::Call { callee, .. } = &instruction.inner {
                            let from = self.translate_instruction_node(name, *bid, iid);
                            let to = self.translate_callee(name, callee)?;

                            edges.push(format!("{from} -> {to};"));
                        }
                    }
                }
            }
        }

        let inner = [subgraphs, edges].concat().join("\n");

        Ok(format!("digraph G {{\n{inner}\n}}"))
    }
}

impl Visualizer {
    #[inline]
    fn get_function_first_instruction(&self, name: &str) -> String {
        self.function_first_instruction
            .get(name)
            .expect("failed to get first instruction from function")
            .clone()
    }

    #[inline]
    fn get_block_first_instruction(&self, name: &str, bid: BlockId) -> String {
        self.block_first_instruction
            .get(&(name.to_string(), bid))
            .expect("failed to get first instruction from block")
            .clone()
    }

    #[inline]
    fn translate_instruction_node(&self, name: &str, bid: BlockId, iid: usize) -> String {
        format!("\"{name}:{bid}:i{iid}\"")
    }

    #[inline]
    fn translate_block_exit_node(&self, name: &str, bid: BlockId) -> String {
        format!("\"{name}:{bid}:exit\"")
    }

    #[inline]
    fn translate_callee(&mut self, name: &str, callee: &Operand) -> Result<String, ()> {
        match callee {
            Operand::Constant(_constant @ Constant::GlobalVariable { name, .. }) => {
                Ok(self.get_function_first_instruction(name))
            }
            Operand::Register {
                rid: _rid @ RegisterId::Temp { bid, iid },
                ..
            } => Ok(self.translate_instruction_node(name, *bid, *iid)),
            _ => todo!("translate_callee: operand {:?}", callee),
        }
    }

    fn translate_function(
        &mut self,
        name: &str,
        signature: &FunctionSignature,
        definition: &FunctionDefinition,
    ) -> Result<String, ()> {
        let mut subgraphs = Vec::new();

        for (bid, block) in &definition.blocks {
            let subgraph = self.translate_block(name, bid, block)?;
            subgraphs.push(subgraph);
        }

        let _unused = self.function_first_instruction.insert(
            name.to_string(),
            self.get_block_first_instruction(name, definition.bid_init),
        );

        let params = signature
            .params
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let label = format!("label=\"fun {} @{} ({})\";", signature.ret, name, params);

        let mut edges = Vec::new();

        // Add edges for block exit
        for (bid, block) in &definition.blocks {
            match &block.exit {
                BlockExit::Jump { arg } => {
                    edges.push(format!(
                        "{} -> {};",
                        self.translate_block_exit_node(name, *bid),
                        self.get_block_first_instruction(name, arg.bid)
                    ));
                }
                BlockExit::ConditionalJump {
                    arg_then, arg_else, ..
                } => {
                    edges.push(format!(
                        "{} -> {} [label=\"true\"];",
                        self.translate_block_exit_node(name, *bid),
                        self.get_block_first_instruction(name, arg_then.bid)
                    ));
                    edges.push(format!(
                        "{} -> {} [label=\"false\"];",
                        self.translate_block_exit_node(name, *bid),
                        self.get_block_first_instruction(name, arg_else.bid)
                    ));
                }
                BlockExit::Switch { default, cases, .. } => {
                    edges.push(format!(
                        "{} -> {} [label=\"default\"];",
                        self.translate_block_exit_node(name, *bid),
                        self.get_block_first_instruction(name, default.bid)
                    ));
                    for (constant, arg) in cases {
                        edges.push(format!(
                            "{} -> {} [label=\"{}\"];",
                            self.translate_block_exit_node(name, *bid),
                            self.get_block_first_instruction(name, arg.bid),
                            constant,
                        ));
                    }
                }
                _ => {}
            }
        }

        // TODO: Add init information (bid_init, allocations)
        let inner = [subgraphs, vec![label], edges].concat().join("\n");

        Ok(format!("subgraph \"cluster.{name}\" {{\n{inner}\n}}"))
    }

    fn translate_block(&mut self, name: &str, bid: &BlockId, block: &Block) -> Result<String, ()> {
        let mut header = Vec::new();
        header.push("style=filled;".to_string());
        header.push("color=lightgrey;".to_string());
        header.push("node [shape=record];".to_string());
        header.push(format!("label=\"{bid}\";"));

        let mut nodes = Vec::new();

        // Add nodes for instructions
        for (iid, instruction) in block.instructions.iter().enumerate() {
            nodes.push(format!(
                "{} [label=\"{}\"]",
                self.translate_instruction_node(name, *bid, iid),
                instruction
            ));
        }

        // Add node for block exit
        nodes.push(format!(
            "{} [label=\"{}\"]",
            self.translate_block_exit_node(name, *bid),
            block.exit
        ));

        let edges = (0..block.instructions.len())
            .map(|iid| self.translate_instruction_node(name, *bid, iid))
            .chain([self.translate_block_exit_node(name, *bid)])
            .collect::<Vec<String>>()
            .join(" -> ");

        let first_instruction = if block.instructions.is_empty() {
            self.translate_block_exit_node(name, *bid)
        } else {
            self.translate_instruction_node(name, *bid, 0)
        };
        let _unused = self
            .block_first_instruction
            .insert((name.to_string(), *bid), first_instruction);

        let inner = [header, nodes, vec![edges]].concat().join("\n");

        Ok(format!("subgraph \"cluster.{name}.{bid}\" {{\n{inner}\n}}"))
    }
}
