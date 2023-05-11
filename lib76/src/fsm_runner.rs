use std::collections::VecDeque;

use crate::fileparsers::msn::{FSMOpcode, FSMStackMachineDefinition, FSM};

#[derive(PartialEq, Debug)]
pub enum FSMRunnerStepResult {
    NotDone,
    DoNextMachine,
}

pub struct FSMRunner {
    stack_machines: Vec<FSMStackMachine>,
}

impl FSMRunner {
    pub fn new(definitions: &[FSMStackMachineDefinition], fsm: &FSM) -> Self {
        Self {
            stack_machines: definitions
                .iter()
                .map(|d| FSMStackMachine::new(d, &fsm.constants))
                .collect(),
        }
    }

    pub fn step_machines(&mut self, fsm: &FSM) {
        for machine in &mut self.stack_machines {
            if machine.halted {
                continue;
            }

            loop {
                let result = machine.step(fsm);
                if result == FSMRunnerStepResult::DoNextMachine {
                    break;
                }
            }
        }
    }
}

struct FSMStackMachine {
    pub constants: Vec<i32>,
    pub start_address: u32,
    pub instruction_pointer: u32,
    pub stack: Vec<i32>,
    pub result_reg: i32,
    pub argument_queue: VecDeque<i32>,
    pub halted: bool,
}

impl FSMStackMachine {
    pub fn new(definition: &FSMStackMachineDefinition, fsm_constants: &[i32]) -> Self {
        let mut constants = Vec::with_capacity(definition.initial_arguments.len());
        for arg in &definition.initial_arguments {
            constants.push(fsm_constants[*arg as usize]);
        }
        Self {
            start_address: definition.start_address,
            constants,
            instruction_pointer: definition.start_address,
            stack: Vec::new(),
            result_reg: 0,
            argument_queue: VecDeque::new(),
            halted: false,
        }
    }

    pub fn step(&mut self, fsm: &FSM) -> FSMRunnerStepResult {
        let instruction = &fsm.raw_instructions[self.instruction_pointer as usize];
        self.instruction_pointer += 1;

        match instruction.opcode {
            FSMOpcode::Push => {
                self.stack.push(instruction.value);
            }
            FSMOpcode::ArgPushS => {
                let s_val = self.stack[(instruction.value - 1) as usize];
                self.argument_queue.push_back(s_val);
            }
            FSMOpcode::ArgPushB => {
                let idx = ((self.constants.len() as i32) + (instruction.value + 1)) as usize;
                let b_val = self.constants[idx as usize];

                self.argument_queue.push_back(b_val);
            }
            FSMOpcode::Adjust => {
                let add_to_sp = instruction.value;

                if add_to_sp < 1 {
                    panic!("FSMOpcode::Adjust What to do when adjusting 0 or negative values?");
                }

                for _ in 0..add_to_sp {
                    self.stack.push(0);
                }
            }
            FSMOpcode::Drop => {
                let sub_from_sp = instruction.value;
                if sub_from_sp < 0 {
                    panic!("FSMOpcode::Drop Expecting positive values");
                }

                for _ in 0..sub_from_sp {
                    self.stack.pop();
                }
            }
            FSMOpcode::Jmp => {
                self.instruction_pointer = instruction.value as u32;
            }
            FSMOpcode::Jz => {
                if self.result_reg == 0 {
                    self.instruction_pointer = instruction.value as u32;
                }
            }
            FSMOpcode::JmpI => {
                self.instruction_pointer = instruction.value as u32;
                return FSMRunnerStepResult::DoNextMachine;
            }
            FSMOpcode::Rst => {
                // TODO: Investigate - probably reload IP and stack ptr
                self.halted = true;
                return FSMRunnerStepResult::DoNextMachine;
            }
            FSMOpcode::Action => {
                let action_name = &fsm.action_table[instruction.value as usize];
                println!("Action {}, args: {:?}", action_name, &self.argument_queue);
                self.argument_queue.clear();
            }
            FSMOpcode::Neg => {
                self.result_reg = -self.result_reg;
            }
        }

        FSMRunnerStepResult::NotDone
    }
}
