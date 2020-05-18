use crate::asm;
use crate::ir;
use crate::ir::HasDtype;
use crate::Translate;

use core::ops::Deref;
use std::collections::HashMap;

struct StackInfo {
    /// Function name for generating the block labels
    name: String,
    /// Minimum stack size required to execute function
    stack_size: usize,
    /// Stack offsets of registers
    stack_offsets_registers: HashMap<ir::RegisterId, usize>,
    /// Stack offset for previous `ra` value
    stack_offset_ra: usize,
    /// Stack offset for previous `s0` value
    stack_offset_s0: usize,
}

#[derive(Default)]
pub struct Asmgen {}

impl Translate<ir::TranslationUnit> for Asmgen {
    type Target = asm::Asm;
    type Error = ();

    fn translate(&mut self, source: &ir::TranslationUnit) -> Result<Self::Target, Self::Error> {
        let mut functions = Vec::new();
        let mut _variables = Vec::new();
        for (name, decl) in &source.decls {
            match decl {
                ir::Declaration::Variable { .. } => todo!(),
                ir::Declaration::Function {
                    signature,
                    definition,
                } => {
                    let definition = definition
                        .as_ref()
                        .expect("function must have its definition");
                    let function = self.translate_function(
                        name.clone(),
                        signature,
                        definition,
                        &source.structs,
                    )?;
                    functions.push(function);
                }
            }
        }

        Ok(asm::Asm {
            unit: asm::TranslationUnit {
                functions,
                variables: _variables,
            },
        })
    }
}

impl Asmgen {
    const STACK_ALIGNMENT: usize = 16;
    const SIZE_OF_S_ZERO: usize = 8;
    const SIZE_OF_RETURN_ADDRESS: usize = 8;

    fn translate_function(
        &self,
        name: String,
        _signature: &ir::FunctionSignature,
        definition: &ir::FunctionDefinition,
        structs: &HashMap<String, Option<ir::Dtype>>,
    ) -> Result<asm::Section<asm::Function>, ()> {
        // TODO: need to map function parameters and memory address
        // Map `RegisterId` and its memory address. The memory address is represented
        // by a distance from the stack pointer.
        let mut stack_offsets_registers = HashMap::new();

        // Allocate memory for return address and s0(frame pointer).
        let mut required_size = Self::SIZE_OF_RETURN_ADDRESS + Self::SIZE_OF_S_ZERO;

        // Allocate memory for local variables.
        for (i, alloc) in definition.allocations.iter().enumerate() {
            required_size += alloc.deref().size_align_of(structs).unwrap().0;
            let rid = ir::RegisterId::local(i);
            assert_eq!(stack_offsets_registers.insert(rid, required_size), None);
        }

        // Allocate memory for saved registers.
        let required_size = definition
            .blocks
            .iter()
            .fold(required_size, |size, (bid, block)| {
                block
                    .instructions
                    .iter()
                    .enumerate()
                    .fold(size, |size, (i, instr)| {
                        let size = size + instr.deref().dtype().size_align_of(structs).unwrap().0;
                        let rid = ir::RegisterId::temp(*bid, i);
                        assert_eq!(stack_offsets_registers.insert(rid, size), None);
                        size
                    })
            });

        // TODO: Allocate memory for next function args

        let stack_size = ((required_size - 1) / Self::STACK_ALIGNMENT + 1) * Self::STACK_ALIGNMENT;
        let stack_offset_ra = stack_size - Self::SIZE_OF_RETURN_ADDRESS;
        let stack_offset_s0 = stack_offset_ra - Self::SIZE_OF_S_ZERO;
        let stack_info = StackInfo {
            name,
            stack_size,
            stack_offsets_registers,
            stack_offset_ra,
            stack_offset_s0,
        };

        let mut function_section = asm::Section::new(Vec::new(), asm::Function::new(Vec::new()));

        self.translate_section_header(&stack_info, &mut function_section)?;
        self.translate_init_block(&stack_info, &mut function_section)?;
        definition
            .blocks
            .iter()
            .map(|(bid, block)| {
                self.translate_block(&stack_info, *bid, block, &mut function_section)
            })
            .collect::<Result<_, _>>()?;
        self.translate_final_block(&stack_info, &mut function_section)?;

        Ok(function_section)
    }

    fn translate_section_header(
        &self,
        stack_info: &StackInfo,
        function_section: &mut asm::Section<asm::Function>,
    ) -> Result<(), ()> {
        // .globl func_name
        function_section
            .header
            .push(asm::Directive::Globl(asm::Label(stack_info.name.clone())));
        // .type func_name, @function
        function_section.header.push(asm::Directive::Type(
            asm::Label(stack_info.name.clone()),
            asm::SymbolType::Function,
        ));

        Ok(())
    }

    fn translate_init_block(
        &self,
        stack_info: &StackInfo,
        function_section: &mut asm::Section<asm::Function>,
    ) -> Result<(), ()> {
        let mut instrs = Vec::new();
        // addi sp, sp, -stack_size
        instrs.push(asm::Instruction::IType {
            instr: asm::IType::ADDI,
            rd: asm::Register::Sp,
            rs1: asm::Register::Sp,
            imm: -(stack_info.stack_size as isize),
        });
        // sd ra, stack_offset_ra(sp)
        instrs.push(asm::Instruction::SType {
            instr: asm::SType::SD,
            rs2: asm::Register::Ra,
            rs1: asm::Register::Sp,
            imm: stack_info.stack_offset_ra as isize,
        });
        // sd s0, stack_offset_s0(sp)
        instrs.push(asm::Instruction::SType {
            instr: asm::SType::SD,
            rs2: asm::Register::S0,
            rs1: asm::Register::Sp,
            imm: stack_info.stack_offset_s0 as isize,
        });
        // addi s0, sp, stack_size
        instrs.push(asm::Instruction::IType {
            instr: asm::IType::ADDI,
            rd: asm::Register::S0,
            rs1: asm::Register::Sp,
            imm: stack_info.stack_size as isize,
        });

        let enter_block = asm::Block::new(Some(asm::Label(stack_info.name.clone())), instrs);
        function_section.body.blocks.push(enter_block);

        Ok(())
    }

    fn translate_final_block(
        &self,
        stack_info: &StackInfo,
        function_section: &mut asm::Section<asm::Function>,
    ) -> Result<(), ()> {
        let mut instrs = Vec::new();
        // ld ra, stack_offset_ra(sp)
        instrs.push(asm::Instruction::IType {
            instr: asm::IType::LD,
            rd: asm::Register::Ra,
            rs1: asm::Register::Sp,
            imm: stack_info.stack_offset_ra as isize,
        });
        // ld s0, stack_offset_s0(sp)
        instrs.push(asm::Instruction::IType {
            instr: asm::IType::LD,
            rd: asm::Register::S0,
            rs1: asm::Register::Sp,
            imm: stack_info.stack_offset_s0 as isize,
        });
        // addi sp, sp, stack_size
        instrs.push(asm::Instruction::IType {
            instr: asm::IType::ADDI,
            rd: asm::Register::Sp,
            rs1: asm::Register::Sp,
            imm: stack_info.stack_size as isize,
        });
        // jr ra
        instrs.push(asm::Instruction::Pseudo(asm::Pseudo::Jr {
            rs: asm::Register::Ra,
        }));

        let exit_label = asm::Label(format!(".{}_END", stack_info.name));
        let exit_block = asm::Block::new(Some(exit_label), instrs);
        function_section.body.blocks.push(exit_block);

        Ok(())
    }

    fn translate_block(
        &self,
        stack_info: &StackInfo,
        bid: ir::BlockId,
        block: &ir::Block,
        function_section: &mut asm::Section<asm::Function>,
    ) -> Result<(), ()> {
        let instrs_for_phinodes = block
            .phinodes
            .iter()
            .map(|p| self.translate_phinode(stack_info, p.deref()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let instrs_for_instructions = block
            .instructions
            .iter()
            .enumerate()
            .map(|(iid, instr)| {
                let dest_rid = ir::RegisterId::temp(bid, iid);
                self.translate_instruction(stack_info, dest_rid, instr.deref())
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let instrs_for_block_exit = self.translate_block_exit(stack_info, &block.exit)?;

        let instrs = vec![
            instrs_for_phinodes,
            instrs_for_instructions,
            instrs_for_block_exit,
        ]
        .into_iter()
        .flatten()
        .collect();

        let label = asm::Label::new(&stack_info.name, bid);
        let block = asm::Block::new(Some(label), instrs);
        function_section.body.blocks.push(block);

        Ok(())
    }

    fn translate_phinode(
        &self,
        _stack_info: &StackInfo,
        _phinode: &ir::Dtype,
    ) -> Result<Vec<asm::Instruction>, ()> {
        todo!()
    }

    fn translate_instruction(
        &self,
        stack_info: &StackInfo,
        dest_rid: ir::RegisterId,
        instruction: &ir::Instruction,
    ) -> Result<Vec<asm::Instruction>, ()> {
        let (mut instrs, rd) = match instruction {
            ir::Instruction::Store { ptr, value } => {
                let instrs = self.translate_store(stack_info, ptr, value)?;
                return Ok(instrs);
            }
            ir::Instruction::Load { ptr } => self.translate_load(stack_info, ptr)?,
            ir::Instruction::Call { callee, args, .. } => self.translate_call(callee, args)?,
            _ => todo!(),
        };

        // Store `rd` into memory
        let dtype = instruction.dtype();
        let dist_s0_to_ptr = stack_info
            .stack_offsets_registers
            .get(&dest_rid)
            .expect("address matched with `rid` must exist");
        let store_instr = asm::SType::store(dtype);

        instrs.push(asm::Instruction::SType {
            instr: store_instr,
            rs2: rd,
            rs1: asm::Register::S0,
            imm: -(*dist_s0_to_ptr as isize),
        });

        Ok(instrs)
    }

    fn translate_store(
        &self,
        stack_info: &StackInfo,
        ptr: &ir::Operand,
        value: &ir::Operand,
    ) -> Result<Vec<asm::Instruction>, ()> {
        let (instrs_for_value, reg_of_value) = self.translate_operand(stack_info, value)?;

        let mut instrs = Vec::new();
        let (rid, dtype) = ptr.get_register().expect("`ptr` must be register");
        let dist_s0_to_ptr = stack_info
            .stack_offsets_registers
            .get(rid)
            .expect("address matched with `rid` must exist");
        let inner_dtype = dtype
            .get_pointer_inner()
            .expect("`dtype` must be pointer type");
        let store_instr = asm::SType::store(inner_dtype.clone());

        instrs.push(asm::Instruction::SType {
            instr: store_instr,
            rs2: reg_of_value,
            rs1: asm::Register::S0,
            imm: -(*dist_s0_to_ptr as isize),
        });

        let instrs = vec![instrs_for_value, instrs]
            .into_iter()
            .flatten()
            .collect();
        Ok(instrs)
    }

    fn translate_load(
        &self,
        stack_info: &StackInfo,
        ptr: &ir::Operand,
    ) -> Result<(Vec<asm::Instruction>, asm::Register), ()> {
        let mut instrs = Vec::new();

        let (rid, dtype) = ptr.get_register().expect("`ptr` must be register");
        let dist_s0_to_ptr = stack_info
            .stack_offsets_registers
            .get(rid)
            .expect("address matched with `rid` must exist");
        let inner_dtype = dtype
            .get_pointer_inner()
            .expect("`dtype` must be pointer type");
        let load_instr = asm::IType::load(inner_dtype.clone());

        // TODO: select register which is not occupied
        let rd = asm::Register::A5;
        instrs.push(asm::Instruction::IType {
            instr: load_instr,
            rd,
            rs1: asm::Register::S0,
            imm: -(*dist_s0_to_ptr as isize),
        });

        Ok((instrs, rd))
    }

    fn translate_call(
        &self,
        callee: &ir::Operand,
        _args: &[ir::Operand],
    ) -> Result<(Vec<asm::Instruction>, asm::Register), ()> {
        let mut instrs = Vec::new();

        // TODO: translate pass the args

        match callee {
            ir::Operand::Constant(constant) => {
                if let ir::Constant::GlobalVariable { name, dtype } = constant {
                    assert!(dtype.get_function_inner().is_some());
                    instrs.push(asm::Instruction::Pseudo(asm::Pseudo::Call {
                        offset: asm::Label(name.clone()),
                    }));
                } else {
                    panic!("`callee` must be `GlobalVariable`")
                }
            }
            _ => todo!(),
        }

        // TODO: select register which is not occupied
        let rd = asm::Register::A5;
        instrs.push(asm::Instruction::Pseudo(asm::Pseudo::Mv {
            rs: asm::Register::A0,
            rd,
        }));

        Ok((instrs, rd))
    }

    fn translate_block_exit(
        &self,
        stack_info: &StackInfo,
        block_exit: &ir::BlockExit,
    ) -> Result<Vec<asm::Instruction>, ()> {
        match block_exit {
            ir::BlockExit::Return { value } => self.translate_return(stack_info, value),
            _ => todo!(),
        }
    }

    fn translate_return(
        &self,
        stack_info: &StackInfo,
        value: &ir::Operand,
    ) -> Result<Vec<asm::Instruction>, ()> {
        let (instrs_for_value, rs) = self.translate_operand(stack_info, value)?;

        let mut instrs = Vec::new();
        instrs.push(asm::Instruction::Pseudo(asm::Pseudo::Mv {
            rs,
            rd: asm::Register::A0,
        }));
        // Jump to exit block
        instrs.push(asm::Instruction::Pseudo(asm::Pseudo::J {
            offset: asm::Label(format!(".{}_END", stack_info.name)),
        }));

        let instrs = vec![instrs_for_value, instrs]
            .into_iter()
            .flatten()
            .collect();
        Ok(instrs)
    }

    fn translate_operand(
        &self,
        stack_info: &StackInfo,
        operand: &ir::Operand,
    ) -> Result<(Vec<asm::Instruction>, asm::Register), ()> {
        match operand {
            ir::Operand::Constant(constant) => self.translate_constant(stack_info, constant),
            ir::Operand::Register { rid, dtype } => self.translate_register(stack_info, rid, dtype),
        }
    }

    fn translate_constant(
        &self,
        _stack_info: &StackInfo,
        constant: &ir::Constant,
    ) -> Result<(Vec<asm::Instruction>, asm::Register), ()> {
        let mut instrs = Vec::new();

        match constant {
            // TODO: consider width and signed option in the future
            ir::Constant::Int { value, .. } => {
                // TODO: select register which is not occupied
                let rd = asm::Register::A5;
                instrs.push(asm::Instruction::Pseudo(asm::Pseudo::Li {
                    rd,
                    imm: *value as isize,
                }));
                Ok((instrs, rd))
            }
            ir::Constant::Undef { .. } => {
                // TODO: select register which is not occupied
                Ok((instrs, asm::Register::A5))
            }
            _ => todo!(),
        }
    }

    fn translate_register(
        &self,
        stack_info: &StackInfo,
        rid: &ir::RegisterId,
        dtype: &ir::Dtype,
    ) -> Result<(Vec<asm::Instruction>, asm::Register), ()> {
        let mut instrs = Vec::new();

        let dist_s0_to_ptr = stack_info
            .stack_offsets_registers
            .get(rid)
            .expect("address matched with `rid` must exist");
        let load_instr = asm::IType::load(dtype.clone());

        // TODO: select register which is not occupied
        let rd = asm::Register::A5;
        instrs.push(asm::Instruction::IType {
            instr: load_instr,
            rd,
            rs1: asm::Register::S0,
            imm: -(*dist_s0_to_ptr as isize),
        });

        Ok((instrs, rd))
    }
}
