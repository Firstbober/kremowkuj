use crate::error;
use std::collections::VecDeque;

// CVM Instructions
#[derive(Copy, Clone, Debug)]
pub enum Instruction {
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
    RowneZ,

    MniejC,
    MniejZ,

    MNrowC,
    MNrowZ,

    // Bitwise operations
    NieB,
    I,
    Lub,
    XLub,
    PrzesunL,
    PrzesunR,

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

#[derive(Debug)]
pub enum Directive {
    CVMAVersion(u64),
    Procedure(u64, String, u64),
    Invalid,
}

// CVM Procedure
#[derive(Clone, Debug)]
pub struct Procedure {
    pub index: u64,
    pub name: String,
    pub parameter_count: u64,
    pub code: VecDeque<Instruction>,
}

#[derive(Clone, Debug)]
pub enum ParseErrorKind {
    NumberEmptyString,
    NumberCannotParse,

    InstructionUnknown,
    DirectiveUnknown,

    InstructionOutsideOfProcedure,
    DirectiveNotEnoughParameters,
}

impl error::Info for error::Error<ParseErrorKind> {
    fn get_message(&self) -> &'static str {
        match self.kind {
            ParseErrorKind::NumberEmptyString => "expected a number, but got nothing instead",
            ParseErrorKind::NumberCannotParse => "parser cannot process this number",
            ParseErrorKind::InstructionUnknown => "this instruction is unknown",
            ParseErrorKind::DirectiveUnknown => "this directive is unknown",
            ParseErrorKind::InstructionOutsideOfProcedure => {
                "instruction is placed outside procedure"
            }
            ParseErrorKind::DirectiveNotEnoughParameters => {
                "directive requires more parameters than inputed"
            }
        }
    }

    fn get_suggestion(&self) -> &'static str {
        match self.kind {
            ParseErrorKind::NumberEmptyString => "enter a number, or fix entered one",
            ParseErrorKind::NumberCannotParse
            | ParseErrorKind::InstructionUnknown
            | ParseErrorKind::DirectiveUnknown => "look at the spec maybe you got something wrong",
            ParseErrorKind::InstructionOutsideOfProcedure => "place it inside the procedure",
            ParseErrorKind::DirectiveNotEnoughParameters => "input required parameters",
        }
    }
}
// Parsed CVMA file
pub struct CVMAFile {
    pub language_version: u64,
    pub procedures: VecDeque<Procedure>,
    pub errors: VecDeque<error::Error<ParseErrorKind>>,
}

pub fn get_number_from_string(
    string: &str,
    use_dec: bool,
    position: &error::Position,
    errors: &mut VecDeque<error::Error<ParseErrorKind>>,
) -> u64 {
    let mut string = string;
    let mut use_dec = use_dec;

    if string.is_empty() {
        errors.push_back(error::Error {
            position: position.clone(),
            kind: ParseErrorKind::NumberEmptyString,
        });
        return 0;
    }

    if let Some(new_string) = string.strip_prefix("d") {
        string = new_string;
        use_dec = true;
    } else if let Some(new_string) = string.strip_prefix("x") {
        string = new_string;
        use_dec = false;
    }

    let result;

    if use_dec {
        result = string.parse::<u64>();
    } else {
        result = u64::from_str_radix(string, 16);
    }

    match result {
        Ok(number) => number,
        Err(_) => {
            errors.push_back(error::Error {
                position: position.clone(),
                kind: ParseErrorKind::NumberEmptyString,
            });
            0
        }
    }
}

pub fn get_instruction_from_strings(
    instruction: &String,
    parameters: &String,
    position: &mut error::Position,
    errors: &mut VecDeque<error::Error<ParseErrorKind>>,
) -> Instruction {
    position.column = (instruction.len() + 1) as i32;

    match instruction.as_str() {
        // Stack
        "PCHNIJ" => {
            Instruction::Pchnij(get_number_from_string(parameters, false, position, errors))
        }
        "USUŃ" => Instruction::Usun,
        "ZMIENNA.K" => {
            Instruction::ZmiennaK(get_number_from_string(parameters, true, position, errors))
        }
        "ZMIENNA.U" => {
            Instruction::ZmiennaU(get_number_from_string(parameters, true, position, errors))
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
        "RÓWNE.Z" => Instruction::RowneZ,

        "MNIEJ.C" => Instruction::MniejC,
        "MNIEJ.Z" => Instruction::MniejZ,

        "MNRÓW.C" => Instruction::MNrowC,
        "MNRÓW.Z" => Instruction::MNrowZ,

        // Bitwise operations
        "NIE.B" => Instruction::NieB,
        "I" => Instruction::I,
        "LUB" => Instruction::Lub,
        "XLUB" => Instruction::XLub,
        "PRZESUŃ.L" => Instruction::PrzesunL,
        "PRZESUŃ.R" => Instruction::PrzesunR,

        // PC register manipulation
        "IDŹDO" => Instruction::IdzDo(get_number_from_string(parameters, false, position, errors)),
        "IDŹDO.ZE" => {
            Instruction::IdzDoZe(get_number_from_string(parameters, false, position, errors))
        }
        "IDŹDO.NZ" => {
            Instruction::IdzDoNz(get_number_from_string(parameters, false, position, errors))
        }
        "WYWOŁAJ" => {
            Instruction::Wywolaj(get_number_from_string(parameters, false, position, errors))
        }
        "STOP" => Instruction::Stop,
        "WRÓĆ" => Instruction::Wroc,

        // Interpreter communication
        "NAT" => Instruction::Nat(get_number_from_string(parameters, false, position, errors)),

        _ => {
            errors.push_back(error::Error {
                position: position.clone(),
                kind: ParseErrorKind::InstructionUnknown,
            });

            Instruction::BrakOperacji
        }
    }
}

pub fn get_directive_from_strings(
    directive: &String,
    parameters: &String,
    position: &mut error::Position,
    errors: &mut VecDeque<error::Error<ParseErrorKind>>,
) -> Directive {
    match directive.as_str() {
        "@CVMA" => {
            let version = get_number_from_string(parameters.as_str(), true, position, errors);
            Directive::CVMAVersion(version)
        }
        "@Procedura" => {
            let params: Vec<&str> = parameters.split("|\"|").collect();

            if params.len() < 3 {
                errors.push_back(error::Error {
                    position: position.clone(),
                    kind: ParseErrorKind::DirectiveNotEnoughParameters,
                });

                return Directive::Procedure(0, String::new(), 0);
            }

            Directive::Procedure(
                get_number_from_string(params[0], false, position, errors),
                params[1].to_owned(),
                get_number_from_string(params[2], false, position, errors),
            )
        }
        _ => {
            errors.push_back(error::Error {
                position: position.clone(),
                kind: ParseErrorKind::DirectiveUnknown,
            });

            Directive::Invalid
        }
    }
}

pub fn read_from_string(content: &str) -> CVMAFile {
    let mut content = content.to_owned();
    content.push('\n');
    let content = content.as_str();

    let mut cvma_file = CVMAFile {
        language_version: 0,
        procedures: VecDeque::new(),
        errors: VecDeque::new(),
    };

    let mut procedure = Procedure {
        index: 0,
        name: String::new(),
        parameter_count: 0,
        code: VecDeque::new(),
    };

    let mut parse_position = error::Position { line: 0, column: 0 };

    {
        let mut instruction: String = String::new();
        let mut parameters: String = String::new();

        let mut is_param = false;
        let mut is_string = false;
        let mut is_string_escape = false;
        let mut is_comment = false;
        let mut is_in_procedure = false;

        let mut idx: i32 = -1;
        let mut line: i32 = -1;

        for character in content.chars() {
            idx += 1;

            if !character.is_alphanumeric()
                && character != '\n'
                && character != '\\'
                && character != '"'
                && character != '.'
                && character != '@'
            {
                if character == ' ' && !instruction.is_empty() {
                    is_param = true;
                }

                if character == ' ' && is_string {
                    parameters.push(character);
                }

                if character == ';' {
                    is_comment = true;
                }

                continue;
            }

            if character == '\n' && is_comment {
                is_comment = false;
            }

            if is_comment {
                continue;
            }

            if character == '\\' && content.chars().nth((idx + 1) as usize).unwrap() == '"' {
                is_string_escape = true;
                continue;
            }

            if character == '"' {
                if is_string_escape {
                    is_string_escape = false;
                    parameters.push(character);
                } else {
                    parameters.push('|');

                    is_string = !is_string;
                    parameters.push(character);

                    parameters.push('|');
                }
                continue;
            }

            if character == '\n' {
                is_param = false;
                line += 1;

                if instruction.is_empty() {
                    continue;
                }

                parse_position.line = (line + 1) as i32;

                if instruction.starts_with("@") {
                    match get_directive_from_strings(
                        &instruction,
                        &parameters,
                        &mut parse_position,
                        &mut cvma_file.errors,
                    ) {
                        Directive::CVMAVersion(version) => cvma_file.language_version = version,
                        Directive::Procedure(idx, name, param_count) => {
                            procedure.index = idx;
                            procedure.name = name;
                            procedure.parameter_count = param_count;

                            is_in_procedure = true;
                        }
                        Directive::Invalid => {}
                    }
                } else {
                    if !is_in_procedure {
                        parse_position.column = 0;

                        cvma_file.errors.push_back(error::Error {
                            position: parse_position.clone(),
                            kind: ParseErrorKind::InstructionOutsideOfProcedure,
                        });
                    }

                    let to_push = get_instruction_from_strings(
                        &instruction,
                        &parameters,
                        &mut parse_position,
                        &mut cvma_file.errors,
                    );

                    procedure.code.push_back(to_push);

                    match to_push {
                        Instruction::Wroc => {
                            is_in_procedure = false;

                            cvma_file.procedures.push_back(procedure.clone());

                            procedure.index = 0;
                            procedure.name = String::new();
                            procedure.parameter_count = 0;
                            procedure.code.clear();
                        }
                        _ => {}
                    }
                }

                instruction.clear();
                parameters.clear();

                continue;
            }

            if is_param {
                parameters.push(character);
            } else {
                instruction.push(character);
            }
        }
    }

    cvma_file
}
