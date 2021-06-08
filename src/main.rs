use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::exit;

#[derive(Copy, Clone, Debug)]
enum Number {
    Integer(u64),
    Floating(f64),
}

impl Number {
    fn force_u64(self: Self) -> u64 {
        match self {
            Self::Integer(int) => int,
            Self::Floating(floating) => floating.floor() as u64,
        }
    }

    fn force_f64(self: Self) -> f64 {
        match self {
            Self::Integer(int) => int as f64,
            Self::Floating(floating) => floating,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Instruction {
    // Stack
    Pchnij(u64),
    Usun,
    ZmiennaK(u64),
    ZmiennaU(u64),

    // Arithemtics
    DodajC,
    DodajZ,

    OdejmC,
    OdejmZ,

    MnozC,
    MnozZ,

    DzielC,
    DzielZ,

    ResztaC,
    ResztaZ,

    JakoCZ,
    JakoZC,

    // Comparisons
    NieL,
    Rowne,

    MniejC,
    MniejZ,

    MNrowC,
    MNrowZ,

    // Bitwise operations
    NieB,
    I,
    Lub,
    XLub,

    // PC register manipulation
    IdzDo(u64),
    IdzDoZe(u64),
    IdzDoNz(u64),
    Wywolaj(u64),
    Wroc,
    Stop,

    // Interpreter communication
    Nat(u64),
    BrakOperacji,
}

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

#[derive(Debug)]
struct CallFrame {
    bottom: u64,
    pc: u64,
    proc: Procedure,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

type NativeProceduresMap =
    HashMap<u64, Box<dyn Fn(&mut VecDeque<Number>, &mut VecDeque<Vec<Number>>)>>;

fn resolve_number(number: &str, interpret_as_dec: bool) -> Result<u64, &str> {
    let mut number = number.to_owned();

    let mut read_as_dec = interpret_as_dec;

    if number.starts_with("d") {
        read_as_dec = true;
        number.remove(0);
    } else if number.starts_with("x") {
        read_as_dec = false;
        number.remove(0);
    }

    let res;

    if read_as_dec {
        res = number.parse::<u64>();
    } else {
        res = u64::from_str_radix(number.as_str(), 16);
    }

    match res {
        Ok(num) => Ok(num),
        Err(_) => Err("Number is not an integer"),
    }
}

fn resolve_num_err(line: String, number: &str, interpret_as_dec: bool) -> u64 {
    match resolve_number(number, interpret_as_dec) {
        Ok(num) => num,
        Err(_) => {
            println!(
                "Error:\n\t{}\n\t^\n\tThis instructions requires valid number. Got {}",
                line, number
            );
            exit(1);
        }
    }
}

fn get_procedures(path: &str) -> Vec<Procedure> {
    let read_lines_res = read_lines(path);

    if read_lines_res.is_err() {
        println!("Error: Invalid path");
        exit(1);
    }

    let mut procedures: Vec<Procedure> = Vec::new();

    let mut procedure_name: String = "".to_owned();
    let mut procedure_index: u64 = 0;
    let mut procedure_param_count: u64 = 0;
    let mut instructions: Vec<Instruction> = Vec::new();

    let mut flag_is_procedure = false;

    for line in read_lines_res.unwrap() {
        let line = line.unwrap();

        if line.is_empty() {
            continue;
        }

        let mut master_split: Vec<&str> =
            line.split(";").nth(0).unwrap().trim().split(" ").collect();

        let instruction = master_split[0];

        if instruction.is_empty() {
            continue;
        }

        match instruction {
            "@Procedura" => {
                master_split.drain(0..1);

                if master_split.len() < 3 {
                    println!("Error:\n\t{}\n\t^\n\t@Procedura requires minimum of three parameters. Got only {}", line, master_split.len());
                    exit(1);
                }

                let index = match resolve_number(master_split[0], false) {
                    Ok(num) => num,
                    Err(_) => {
                        println!(
                            "Error:\n\t{}\n\t^\n\t@Procedura requires valid numer. Got {}",
                            line, master_split[0]
                        );
                        exit(1);
                    }
                };

                let cloned_ms = master_split.clone();
                let parameter_count = match resolve_number(cloned_ms.last().unwrap(), true) {
                    Ok(num) => num,
                    Err(_) => {
                        println!(
                            "Error:\n\t{}\n\t^\n\t@Procedura requires valid numer. Got {}",
                            line,
                            cloned_ms.last().unwrap()
                        );
                        exit(1);
                    }
                };

                master_split.remove(0);
                master_split.remove(master_split.len() - 1);

                let mut name = master_split.join(" ");
                name.remove(0);
                name.remove(name.len() - 1);
                name = name.replace("\\\"", "\"");

                if name.is_empty() {
                    println!("Error:\n\t{}\n\t^\n\t@Procedura requires name", line);
                    exit(1);
                }

                let mut reserved_index = false;

                for procedure in procedures.as_slice() {
                    if procedure.index == index {
                        reserved_index = true;
                    }
                }

                if reserved_index {
                    println!(
                        "Error:\n\t{}\n\t^\n\tIndex for this procedure is already in use",
                        line
                    );
                    exit(1);
                }

                procedure_name = name;
                procedure_index = index;
                procedure_param_count = parameter_count;

                flag_is_procedure = true;
                continue;
            }
            "WRÓĆ" => {
                flag_is_procedure = false;

                instructions.push(Instruction::Wroc);
                let cloned_instructions = instructions.clone();

                procedures.push(Procedure {
                    index: procedure_index,
                    name: procedure_name.to_owned(),
                    parameter_count: procedure_param_count,
                    code: cloned_instructions,
                });
                instructions.clear();
            }

            _ => {}
        }

        if flag_is_procedure {
            let instr: Instruction = match instruction {
                // Stack
                "PCHNIJ" => {
                    Instruction::Pchnij(resolve_num_err(line.clone(), master_split[1], false))
                }
                "USUŃ" => Instruction::Usun,
                "ZMIENNA.K" => {
                    Instruction::ZmiennaK(resolve_num_err(line.clone(), master_split[1], false))
                }
                "ZMIENNA.U" => {
                    Instruction::ZmiennaU(resolve_num_err(line.clone(), master_split[1], false))
                }

                // Arithemtics
                "DODAJ.C" => Instruction::DodajC,
                "DODAJ.Z" => Instruction::DodajZ,

                "ODEJM.C" => Instruction::OdejmC,
                "ODEJM.Z" => Instruction::OdejmZ,

                "MNÓŻ.C" => Instruction::MnozC,
                "MNÓŻ.Z" => Instruction::MnozZ,

                "DZIEL.C" => Instruction::DzielC,
                "DZIEL.Z" => Instruction::DzielZ,

                "RESZTA.C" => Instruction::ResztaC,
                "RESZTA.Z" => Instruction::ResztaZ,

                "JAKO.CZ" => Instruction::JakoCZ,
                "JAKO.ZC" => Instruction::JakoZC,

                // Comparisons
                "NIE.L" => Instruction::NieL,
                "RÓWNE" => Instruction::Rowne,

                "MNIEJ.C" => Instruction::MniejC,
                "MNIEJ.Z" => Instruction::MniejZ,

                "MNRÓW.C" => Instruction::MNrowC,
                "MNRÓW.Z" => Instruction::MNrowZ,

                // Bitwise operations
                "NIE.B" => Instruction::NieB,
                "I" => Instruction::I,
                "LUB" => Instruction::Lub,
                "XLUB" => Instruction::XLub,

                // PC register manipulation
                "IDŹDO" => {
                    Instruction::IdzDo(resolve_num_err(line.clone(), master_split[1], false))
                }
                "IDŹDO.ZE" => {
                    Instruction::IdzDoZe(resolve_num_err(line.clone(), master_split[1], false))
                }
                "IDŹDO.NZ" => {
                    Instruction::IdzDoNz(resolve_num_err(line.clone(), master_split[1], false))
                }
                "WYWOŁAJ" => {
                    Instruction::Wywolaj(resolve_num_err(line.clone(), master_split[1], false))
                }
                "STOP" => Instruction::Stop,

                // Interpreter communication
                "NAT" => Instruction::Nat(resolve_num_err(line.clone(), master_split[1], false)),

                _ => Instruction::BrakOperacji,
            };

            instructions.push(instr);
        }
    }

    procedures
}

fn push_call(call_stack: &mut VecDeque<CallFrame>, frame: CallFrame) {
    call_stack.push_back(frame);
}

fn pop_call(call_stack: &mut VecDeque<CallFrame>) -> CallFrame {
    call_stack.pop_back().unwrap()
}

macro_rules! cvm_arithmetics_u64 {
    ($stack:expr, $op:tt) => {
        let y = $stack.pop_back().unwrap().force_u64();
        let x = $stack.pop_back().unwrap().force_u64();
        $stack.push_back(Number::Integer(x $op y));
        ()
    };
}

macro_rules! cvm_arithmetics_f64 {
    ($stack:expr, $op:tt) => {
        let y = $stack.pop_back().unwrap().force_f64();
        let x = $stack.pop_back().unwrap().force_f64();
        $stack.push_back(Number::Floating(x $op y));
        ()
    };
}

fn execute_procedure(
    procedure: &Procedure,
    procedures: &Vec<Procedure>,
    stack: &mut VecDeque<Number>,
    bottom: u64,
    call_stack: &mut Vec<CallFrame>,
    native_procedures: &NativeProceduresMap,
    allocation_array: &mut VecDeque<Vec<Number>>,
) {
    let mut pc: u64 = 0;
    let mut continue_execution = true;
    let mut bottom = bottom;

    while continue_execution {
        let instruction = procedure.code[pc as usize];
        pc += 1;

        match instruction {
            // Stack
            Instruction::Pchnij(value) => {
                stack.push_back(Number::Integer(value));
            }
            Instruction::Usun => drop(stack.pop_back()),
            Instruction::ZmiennaK(index) => {
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

            Instruction::JakoCZ => {
                let num = stack.pop_back().unwrap();

                match num {
                    Number::Integer(int) => stack.push_back(Number::Floating(int as f64)),
                    Number::Floating(_) => stack.push_back(num),
                }
            }
            Instruction::JakoZC => {
                let num = stack.pop_back().unwrap();

                match num {
                    Number::Integer(_) => stack.push_back(num),
                    Number::Floating(floating) => stack.push_back(Number::Integer(floating as u64)),
                }
            }

            // Comparisons
            Instruction::NieL => {
                let x = stack.pop_back().unwrap();
                if x.force_u64() == 0 {
                    stack.push_back(Number::Integer(1));
                } else {
                    stack.push_back(Number::Integer(0));
                }
            }
            Instruction::Rowne => {
                let y = stack.pop_back().unwrap();
                let x = stack.pop_back().unwrap();
                stack.push_back(Number::Integer((x.force_u64() == y.force_u64()) as u64));
            }

            Instruction::MniejC => {
                let y = stack.pop_back().unwrap().force_u64();
                let x = stack.pop_back().unwrap().force_u64();
                stack.push_back(Number::Integer((x < y) as u64));
            }
            Instruction::MniejZ => {
                let y = stack.pop_back().unwrap().force_f64();
                let x = stack.pop_back().unwrap().force_f64();
                stack.push_back(Number::Integer((x < y) as u64));
            }

            Instruction::MNrowC => {
                let y = stack.pop_back().unwrap().force_u64();
                let x = stack.pop_back().unwrap().force_u64();
                stack.push_back(Number::Integer((x <= y) as u64));
            }
            Instruction::MNrowZ => {
                let y = stack.pop_back().unwrap().force_f64();
                let x = stack.pop_back().unwrap().force_f64();
                stack.push_back(Number::Integer((x <= y) as u64));
            }

            // Bitwise operations
            Instruction::NieB => {
                let x = stack.pop_back().unwrap().force_u64();
                stack.push_back(Number::Integer(!x));
            }
            Instruction::I => {
                let y = stack.pop_back().unwrap().force_u64();
                let x = stack.pop_back().unwrap().force_u64();
                stack.push_back(Number::Integer(x & y));
            }
            Instruction::Lub => {
                let y = stack.pop_back().unwrap().force_u64();
                let x = stack.pop_back().unwrap().force_u64();
                stack.push_back(Number::Integer(x | y));
            }
            Instruction::XLub => {
                let y = stack.pop_back().unwrap().force_u64();
                let x = stack.pop_back().unwrap().force_u64();
                stack.push_back(Number::Integer(x ^ y));
            }

            // PC register manipulation
            Instruction::IdzDo(new_pc) => {
                pc -= 1;
                pc = new_pc;
            }
            Instruction::IdzDoZe(new_pc) => {
                let x = stack.pop_back().unwrap().force_u64();
                pc -= 1;

                if x == 0 {
                    pc = new_pc;
                }
            }
            Instruction::IdzDoNz(new_pc) => {
                let x = stack.pop_back().unwrap().force_u64();
                pc -= 1;

                if x != 0 {
                    pc = new_pc;
                }
            }
            Instruction::Wywolaj(proc_idx) => {
                /*
                push_call(
                    call_stack,
                    CallFrame {
                        bottom: bottom,
                        pc: pc,
                        proc: procedure.clone(),
                    },
                );
                */

                let mut new_proc: Option<&Procedure> = None;

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
                            bottom + new_proc.parameter_count,
                            call_stack,
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

    if bottom != 0 {
        //pop_call(call_stack);

        while stack.len() > bottom as usize {
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

fn peek(stack: &VecDeque<Number>, n: usize) -> Number {
    *stack.get((stack.len() - 1) - n).unwrap()
}

fn register_natproc_io(native_procedures: &mut NativeProceduresMap) {
    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::PutC as u64,
        |stack, _| {
            print!("{}", peek(stack, 0).force_u64());
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::PutZ as u64,
        |stack, _| {
            print!("{}", peek(stack, 0).force_f64());
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::PutU as u64,
        |stack, _| {
            print!(
                "{}",
                char::from_u32(peek(stack, 0).force_u64() as u32).unwrap()
            );
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::GetC as u64,
        |stack, _| {
            match get_stdin_input().trim().parse::<u64>() {
                Ok(value) => {
                    stack.push_back(Number::Integer(value));
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
                    stack.push_back(Number::Floating(value));
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

            stack.push_back(Number::Integer(input.chars().nth(0).unwrap() as u64));
        }
    );
}
fn register_natproc_memory(native_procedures: &mut NativeProceduresMap) {
    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Alloc as u64,
        |stack, alloc_array| {
            let to_alloc = peek(stack, 0).force_u64();

            let mut blocks: Vec<Number> = Vec::new();

            for _ in 0..to_alloc {
                blocks.push(Number::Integer(0));
            }

            alloc_array.push_back(blocks);

            stack.push_back(Number::Integer((alloc_array.len() - 1) as u64));
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Free as u64,
        |stack, alloc_array| {
            let alloc_addr = peek(stack, 0).force_u64();
            alloc_array.remove(alloc_addr as usize);
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Read as u64,
        |stack, alloc_array| {
            let idx = peek(stack, 0).force_u64() as usize;
            let addr = peek(stack, 1).force_u64() as usize;

            stack.push_back(alloc_array[addr][idx]);
        }
    );

    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Write as u64,
        |stack, alloc_array| {
            let value = peek(stack, 0);
            let idx = peek(stack, 1).force_u64() as usize;
            let addr = peek(stack, 2).force_u64() as usize;

            alloc_array[addr][idx] = value;
        }
    );
}
fn register_natproc_strings(native_procedures: &mut NativeProceduresMap) {
    register_native_procedure!(
        native_procedures,
        ReservedNativeProcedures::Print as u64,
        |stack, alloc_array| {
            let addr = peek(stack, 0).force_u64() as usize;
            let mut string: String = String::new();

            for block in alloc_array[addr].clone() {
                let bytes = block.force_u64().to_ne_bytes();
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

    let mut stack: VecDeque<Number> = VecDeque::new();
    let mut call_stack: Vec<CallFrame> = Vec::new();
    let mut allocation_array: VecDeque<Vec<Number>> = VecDeque::new();

    let mut native_procedures: NativeProceduresMap = HashMap::new();

    register_natproc_io(&mut native_procedures);
    register_natproc_memory(&mut native_procedures);
    register_natproc_strings(&mut native_procedures);

    let procedures = get_procedures(args[1].as_str());
    let mut has_main_procedure = false;

    for procedure in procedures.as_slice() {
        if procedure.index == 0 {
            has_main_procedure = true;
            execute_procedure(
                procedure,
                &procedures,
                &mut stack,
                0,
                &mut call_stack,
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
