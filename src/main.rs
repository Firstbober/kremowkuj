use libkrem::parse::Instruction;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::process::exit;

#[repr(u64)]
enum ReservedNativeProcedures {
    // 0x - I/O
    PutC = 0x00,
    PutZ = 0x01,
    PutU = 0x02,
    GetC = 0x03,
    GetZ = 0x04,
    GetU = 0x05,

    // 1x - memory
    Alloc = 0x10,
    Free = 0x11,
    Read = 0x12,
    Write = 0x13,

    // 2x - strings
    Print = 0x20,
}

#[derive(Clone, Debug)]
struct Procedure {
    index: u64,
    name: String,
    parameter_count: u64,
    code: Vec<Instruction>,
}

type NativeProceduresMap = HashMap<u64, Box<dyn Fn(&mut VecDeque<u64>, &mut VecDeque<Vec<u64>>)>>;

macro_rules! cvm_arithmetics_u64 {
    ($stack:expr, $op:tt) => {
        unsafe {
            let y = std::mem::transmute::<u64, i64>($stack.pop_back().unwrap());
            let x = std::mem::transmute::<u64, i64>($stack.pop_back().unwrap());
            $stack.push_back(std::mem::transmute::<i64, u64>(x $op y));
        }
        ()
    };
}

macro_rules! cvm_arithmetics_f64 {
    ($stack:expr, $op:tt) => {
        let y = f64::from_bits($stack.pop_back().unwrap());
        let x = f64::from_bits($stack.pop_back().unwrap());
        $stack.push_back((x $op y).to_bits());
        ()
    };
}

fn execute_procedure(
    procedure: &libkrem::parse::Procedure,
    procedures: &VecDeque<libkrem::parse::Procedure>,
    stack: &mut VecDeque<u64>,
    bottom: u64,
    native_procedures: &NativeProceduresMap,
    allocation_array: &mut VecDeque<Vec<u64>>,
) {
    let mut pc: u64 = 0;
    let mut continue_execution = true;
    let bottom = bottom;

    while continue_execution {
        let instruction = procedure.code[pc as usize];
        pc += 1;

        match instruction {
            // Stack
            Instruction::Pchnij(value) => {
                stack.push_back(value);
            }
            Instruction::Usun => drop(stack.pop_back()),
            Instruction::ZmiennaK(index) => {
                println!("{:?}, {}", stack, bottom);
                stack.push_back(stack[(bottom + index) as usize]);
            }
            Instruction::ZmiennaU(index) => {
                let x = stack.pop_back().unwrap();
                stack[(bottom + index) as usize] = x;
            }

            // Arithemtics
            Instruction::DodajC => {
                cvm_arithmetics_u64!(stack, +);
            }
            Instruction::DodajZ => {
                cvm_arithmetics_f64!(stack, +);
            }

            Instruction::OdejmC => {
                cvm_arithmetics_u64!(stack, -);
            }
            Instruction::OdejmZ => {
                cvm_arithmetics_f64!(stack, -);
            }

            Instruction::MnozC => {
                cvm_arithmetics_u64!(stack, *);
            }
            Instruction::MnozZ => {
                cvm_arithmetics_f64!(stack, *);
            }

            Instruction::DzielC => {
                cvm_arithmetics_u64!(stack, /);
            }
            Instruction::DzielZ => {
                cvm_arithmetics_f64!(stack, /);
            }

            Instruction::ResztaC => {
                cvm_arithmetics_u64!(stack, %);
            }
            Instruction::ResztaZ => {
                cvm_arithmetics_f64!(stack, %);
            }

            Instruction::JakoCZ => unsafe {
                let num = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());

                stack.push_back((num as f64).to_bits());
            },
            Instruction::JakoZC => {
                let num = stack.pop_back().unwrap();

                stack.push_back(f64::from_bits(num).floor() as u64);
            }

            // Comparisons
            Instruction::NieL => unsafe {
                let x = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());
                if x == 0 {
                    stack.push_back(1);
                } else {
                    stack.push_back(0);
                }
            },
            Instruction::Rowne => unsafe {
                let y = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());
                let x = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());
                stack.push_back((x == y) as u64);
            },
            Instruction::RowneZ => {
                let y = f64::from_bits(stack.pop_back().unwrap());
                let x = f64::from_bits(stack.pop_back().unwrap());
                stack.push_back((x == y) as u64);
            }

            Instruction::MniejC => unsafe {
                let y = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());
                let x = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());
                stack.push_back((x < y) as u64);
            },
            Instruction::MniejZ => {
                let y = stack.pop_back().unwrap();
                let x = stack.pop_back().unwrap();
                stack.push_back((x < y) as u64);
            }

            Instruction::MNrowC => unsafe {
                let y = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());
                let x = std::mem::transmute::<u64, i64>(stack.pop_back().unwrap());
                stack.push_back((x <= y) as u64);
            },
            Instruction::MNrowZ => {
                let y = f64::from_bits(stack.pop_back().unwrap());
                let x = f64::from_bits(stack.pop_back().unwrap());
                stack.push_back((x <= y) as u64);
            }

            // Bitwise operations
            Instruction::NieB => {
                let x = stack.pop_back().unwrap();
                stack.push_back(!x);
            }
            Instruction::I => {
                let y = stack.pop_back().unwrap();
                let x = stack.pop_back().unwrap();
                stack.push_back(x & y);
            }
            Instruction::Lub => {
                let y = stack.pop_back().unwrap();
                let x = stack.pop_back().unwrap();
                stack.push_back(x | y);
            }
            Instruction::XLub => {
                let y = stack.pop_back().unwrap();
                let x = stack.pop_back().unwrap();
                stack.push_back(x ^ y);
            }
            Instruction::PrzesunL => {
                let y = stack.pop_back().unwrap();
                let x = stack.pop_back().unwrap();
                stack.push_back(x << y);
            }
            Instruction::PrzesunR => {
                let y = stack.pop_back().unwrap();
                let x = stack.pop_back().unwrap();
                stack.push_back(x >> y);
            }

            // PC register manipulation
            Instruction::IdzDo(new_pc) => {
                pc = new_pc;
            }
            Instruction::IdzDoZe(new_pc) => {
                let x = stack.pop_back().unwrap();

                if x == 0 {
                    pc = new_pc;
                }
            }
            Instruction::IdzDoNz(new_pc) => {
                let x = stack.pop_back().unwrap();

                if x != 0 {
                    pc = new_pc;
                }
            }
            Instruction::Wywolaj(proc_idx) => {
                let mut new_proc: Option<&libkrem::parse::Procedure> = None;

                for procedure in procedures {
                    if procedure.index == proc_idx {
                        new_proc = Some(procedure);
                    }
                }

                match new_proc {
                    Some(new_proc) => {
                        if new_proc.code.len() == 0 {
                            continue;
                        }

                        execute_procedure(
                            new_proc,
                            procedures,
                            stack,
                            (stack.len() - (new_proc.parameter_count as usize)) as u64,
                            native_procedures,
                            allocation_array,
                        );
                    }
                    None => {
                        println!(
                            "Runtime error: Procedure with index {} does not exist",
                            proc_idx
                        );
                        exit(1);
                    }
                }
            }
            Instruction::Wroc | Instruction::Stop => {
                continue_execution = false;
            }

            // Interpreter communication
            Instruction::Nat(nat_proc) => {
                if !native_procedures.contains_key(&nat_proc) {
                    println!(
                        "Runtime error: Native procedure {:X} does not exist",
                        nat_proc
                    );
                    exit(1);
                }

                native_procedures.get(&nat_proc).unwrap()(stack, allocation_array);
            }
            Instruction::BrakOperacji => unimplemented!(),
        }
    }

    if procedure.index != 0 {
        while stack.len() > (bottom + 1) as usize {
            stack.pop_back();
        }
    }
}

macro_rules! register_native_procedure {
    ($natprocs:expr, $idx:expr, $procedure:expr) => {
        $natprocs.insert($idx, Box::new($procedure));
    };
}

fn get_stdin_input() -> String {
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Runtime error: stdin failed");
    buffer
}

fn peek(stack: &VecDeque<u64>, n: usize) -> u64 {
    *stack.get((stack.len() - 1) - n).unwrap()
}

fn register_natproc_io(native_procedures: &mut NativeProceduresMap) {
    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::PutC as u64,
        |stack, _| {
            print!("{}", peek(stack, 0));
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::PutZ as u64,
        |stack, _| {
            print!("{}", f64::from_bits(peek(stack, 0)));
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::PutU as u64,
        |stack, _| {
            print!("{}", char::from_u32(peek(stack, 0) as u32).unwrap());
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::GetC as u64,
        |stack, _| {
            match get_stdin_input().trim().parse::<u64>() {
                Ok(value) => {
                    stack.push_back(value);
                }
                Err(_) => {
                    println!("Runtime error: got invalid input, expected u64");
                    exit(1);
                }
            }
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::GetZ as u64,
        |stack, _| {
            match get_stdin_input().trim().parse::<f64>() {
                Ok(value) => {
                    stack.push_back(value.to_bits());
                }
                Err(_) => {
                    println!("Runtime error: got invalid input, expected f64");
                    exit(1);
                }
            }
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::GetU as u64,
        |stack, _| {
            let mut input = get_stdin_input();

            while input.is_empty() {
                input = get_stdin_input();
            }

            stack.push_back(input.chars().nth(0).unwrap() as u64);
        }
    );
}
fn register_natproc_memory(native_procedures: &mut NativeProceduresMap) {
    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Alloc as u64,
        |stack, alloc_array| {
            let to_alloc = peek(stack, 0);

            let mut blocks: Vec<u64> = Vec::new();

            for _ in 0..to_alloc {
                blocks.push(0);
            }

            alloc_array.push_back(blocks);

            stack.push_back((alloc_array.len() - 1) as u64);
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Free as u64,
        |stack, alloc_array| {
            let alloc_addr = peek(stack, 0);
            alloc_array.remove(alloc_addr as usize);
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Read as u64,
        |stack, alloc_array| {
            let idx = peek(stack, 0) as usize;
            let addr = peek(stack, 1) as usize;

            stack.push_back(alloc_array[addr][idx]);
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Write as u64,
        |stack, alloc_array| {
            let value = peek(stack, 0);
            let idx = peek(stack, 1) as usize;
            let addr = peek(stack, 2) as usize;

            alloc_array[addr][idx] = value;
        }
    );
}
fn register_natproc_strings(native_procedures: &mut NativeProceduresMap) {
    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Print as u64,
        |stack, alloc_array| {
            let addr = peek(stack, 0) as usize;
            let mut string: String = String::new();

            for block in alloc_array[addr].clone() {
                let bytes = block.to_ne_bytes();
                let tmp_str = std::str::from_utf8(&bytes).unwrap();

                for ch in tmp_str.chars() {
                    if ch == '\0' {
                        break;
                    }

                    string.push(ch);
                }
            }

            print!("{}", string);
        }
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut flag_show_dbg = false;

    if args.len() < 2 {
        println!("Usage: {} file [--dbg]", args[0]);
        exit(1);
    }

    if args.contains(&"--dbg".to_owned()) {
        flag_show_dbg = true;
    }

    let mut stack: VecDeque<u64> = VecDeque::new();
    let mut allocation_array: VecDeque<Vec<u64>> = VecDeque::new();

    let mut native_procedures: NativeProceduresMap = HashMap::new();

    register_natproc_io(&mut native_procedures);
    register_natproc_memory(&mut native_procedures);
    register_natproc_strings(&mut native_procedures);

    let content = fs::read_to_string(args[1].as_str()).unwrap();
    let content = content.as_str();

    let cvma_file = libkrem::parse::read_from_string(content);

    if cvma_file.errors.len() > 0 {
        libkrem::error_print::print_errors(
            "parsing error",
            args[1].as_str(),
            content,
            cvma_file.errors,
        );
        exit(1);
    }

    let procedures = &cvma_file.procedures;
    let mut has_main_procedure = false;

    for procedure in procedures.clone() {
        if procedure.index == 0 {
            has_main_procedure = true;
            execute_procedure(
                &procedure,
                &procedures,
                &mut stack,
                0,
                &native_procedures,
                &mut allocation_array,
            );
        }
    }

    if !has_main_procedure {
        println!("Error: Main procedure is not defined");
        exit(1);
    }

    if flag_show_dbg {
        println!("============\nValue stack: {:?}", stack);
        println!("Allocation array: {:?}", allocation_array);
    }
}
