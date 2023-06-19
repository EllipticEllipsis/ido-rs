use std::fmt;
use std::mem;

use crate::mips::*;

const BINASM_RECORD_LENGTH: usize = 0x10;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum ArgFormat {
    frob {
        symno: i32,
        reg: register,
        offset: i16,
        base: register,
        mem_tag: u32,
    }, // reg, offset(+/-32k), base
    fra {
        symno: i32,
        reg: register,
        offset: i32,
    }, // reg, [sym]+offset
    fri {
        reg: register,
        immediate: i32,
        mem_tag: u32,
    }, // reg, immed (32 bit)
    frrr {
        reg1: register,
        reg2: register,
        reg3: register,
    }, // reg, reg, reg
    frri {
        reg1: register,
        reg2: register,
        immediate: i32,
        mem_tag: u32,
    }, // reg, reg, immed (32 bit)
    frr {
        reg1: register,
        reg2: register,
    }, // reg, reg
    fa {
        symno: i32,
        base: register,
        immediate: i32,
        mem_tag: u32,
    }, // [sym]+offset [+(base)]
    fr {
        reg: register,
    }, // reg
    frrl {
        symno: i32,
        reg1: register,
        reg2: register,
    }, // reg, reg, sym
    frl {
        symno: i32,
        reg: register,
    }, // reg, sym
    fl {
        symno: i32,
    }, // sym
    forrr, // co processor if required?
    fril {
        symno: i32,
        reg: register,
        immediate: i32,
        mem_tag: u32,
    }, // reg, immed, label
    fi {
        immediate: i32,
    }, // immed
    foa,   // op, address
    frrrr {
        reg1: register,
        reg2: register,
        reg3: register,
        reg4: register,
    }, // reg, reg, reg, reg
}

// impl fmt::Display for ArgFormat {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             ArgFormat::frob {
//                 symno,
//                 reg,
//                 offset,
//                 base,
//                 mem_tag: _,
//             } => {
//                 write!(f, "{reg}, {offset}({base})")
//             }
//             _ => write!(f, "{:?}", self),
//         }
//     }
// }

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Asm {
    ilabel {
        symno: i32,
    },
    isym,
    iglobal {
        symno: i32,
    },
    icpload {
        symno: i32,
        args: ArgFormat,
    },
    ialign {
        length: u32,
    },
    iascii {
        length: u32,
        repeat: u32,
        string: AsciiString,
    },
    iasciiz {
        length: u32,
        repeat: u32,
        string: AsciiString,
    },
    ibyte {
        symno: i32,
        expression: i32,
        repeat: u32,
    },
    icomm {
        symno: i32,
        length: u32,
    },
    ilcomm {
        symno: i32,
        length: u32,
    },
    idata {
        symno: i32,
    },
    idouble {
        length: u32,
        repeat: u32,
        string: FPString,
    },
    ifile {
        symno: i32,
        length: u32,
        repeat: u32,
        string: AsciiString,
    },
    ifloat {
        length: u32,
        repeat: u32,
        string: FPString,
    },
    ihalf {
        symno: i32,
        expression: i32,
        repeat: u32,
    },
    icprestore {
        offset: u32,
    }, // could be i32, maybe
    igpword {
        symno: i32,
        expression: i32,
        repeat: u32,
    },
    icpadd {
        symno: i32,
        args: ArgFormat,
    },
    iweakext {
        symno: i32,
        lexlev: i32,
    },
    iloopno,
    ispace,
    itext, // May be more to this
    iword {
        symno: i32,
        expression: i32,
        repeat: u32,
    },
    iocode {
        op: asmcode,
        args: ArgFormat,
    }, // more needed
    iend {
        symno: i32,
    },
    isdata {
        symno: i32,
    },
    irdata {
        symno: i32,
    },
    ient {
        symno: i32,
        lexlev: i32,
    },
    iloc {
        filenumber: u32,
        linenumber: u32,
    },
    ibgnb {
        symno: i32,
    },
    iendb {
        symno: i32,
    },
    iasm0 {
        symno: i32,
    },
    iset {
        value: set_value,
    },
    icpalias,
    irep,
    iendrep {
        symno: i32,
    },
    ilab {
        symno: i32,
    },
    ivreg,
    imask {
        regmask: u32,
        regoffset: i32,
    },
    ifmask {
        regmask: u32,
        regoffset: i32,
    },
    ierr,
    iglobabs,
    iverstamp {
        majornumber: i32,
        minornumber: i32,
    },
    iframe {
        frameoffset: i32,
        framereg: register,
        pcreg: register,
    },
    iextended,
    iextern,
    iaent {
        symno: i32,
        lexlev: i32,
    },
    ioption {
        opt: opt_type,
        value: OptValue,
    },
    inoalias,
    ialias,
    imtag,
    imalias,
    istruct,
    ilivereg {
        gpmask: u32,
        fpmask: u32,
    },
    igjaldef,
    igjallive,
    igjrlive,
    ishift_addr,
    irestext {
        symno: i32,
    },
    idword {
        expression: i32,
        repeat: u32,
    },
    iprologue {
        symno: i32,
        lexlev: i32,
    },
    iedata,
    ialloc {
        symno: i32,
    },
}

#[derive(Debug)]
struct AsciiString {
    s: Vec<u8>,
}

// Floating point string, no ""
#[derive(Debug)]
struct FPString {
    s: Vec<u8>,
}

impl fmt::Display for AsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for c in &self.s {
            if 0x20 <= *c && *c <= 0x7E {
                write!(f, "{}", *c as char)?;
            } else {
                write!(f, "\\X{c:02X}")?;
            }
        }
        write!(f, "\"")
    }
}
impl fmt::Display for FPString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in &self.s {
            if 0x20 <= *c && *c <= 0x7E {
                write!(f, "{}", *c as char)?
            } else {
                write!(f, "\\x{c:02X}")?
            }
        }
        write!(f, "")
    }
}

impl fmt::Display for Asm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Asm::ilabel { symno } => {
                if *symno > 0 {
                    // TODO: read symbol table
                    write!(f, "SYM_{}:", *symno)
                } else {
                    write!(f, "${}:", -*symno)
                }
            }
            Asm::ialign { length } => {
                write!(f, "\t.align\t{length}")
            }
            Asm::iascii {
                length: _,
                repeat: _,
                string,
            } => {
                write!(f, "\t.ascii\t{string}")
            }
            Asm::iasciiz {
                length: _,
                repeat: _,
                string,
            } => {
                write!(f, "\t.asciiz\t{string}")
            }
            Asm::ifile {
                symno,
                length: _,
                repeat: _,
                string,
            } => {
                write!(f, "\t.file\t{symno} {string}")
            }
            Asm::ifloat {
                length: _,
                repeat,
                string,
            } => {
                write!(f, "\t.float\t{string}:{repeat}")
            }
            Asm::idouble {
                length: _,
                repeat,
                string,
            } => {
                write!(f, "\t.double\t{string}:{repeat}")
            }
            Asm::iframe {
                frameoffset,
                framereg,
                pcreg,
            } => write!(f, "\t.frame\t{framereg}, {frameoffset}, {pcreg}"),
            Asm::iverstamp {
                majornumber,
                minornumber,
            } => write!(f, "\t.verstamp\t{majornumber} {minornumber}"),
            Asm::iloc {
                filenumber,
                linenumber,
            } => write!(f, "\t.loc\t{filenumber} {linenumber}"),
            Asm::ient { symno, lexlev } => write!(f, "\t.ent\tSYM_{symno} {lexlev}"), // TODO read symbol table
            Asm::ilivereg { gpmask, fpmask } => {
                write!(f, "\t.livereg\t0x{gpmask:08X},0x{fpmask:08X}")
            }
            Asm::iglobal { symno } => write!(f, "\t.globl\tSYM_{symno}"), // TODO read symbol table
            Asm::itext => write!(f, "\t.text\t"), // Strangely listed separately
            Asm::idata { symno: _ } => write!(f, "\t.data\t"),
            Asm::isdata { symno: _ } => write!(f, "\t.sdata\t"),
            Asm::irdata { symno: _ } => write!(f, "\t.rdata\t"),
            Asm::iset { value } => write!(f, "\t.set\t {}", value),
            Asm::idword { expression, repeat } => {
                write!(f, "\t.word\t{expression} : {repeat} # dword")
            }
            Asm::ibyte {
                symno: _,
                expression,
                repeat,
            } => write!(f, "\t.byte\t{expression} : {repeat}"),
            Asm::ihalf {
                symno: _,
                expression,
                repeat,
            } => write!(f, "\t.half\t{expression} : {repeat}"),
            Asm::iword {
                symno,
                expression,
                repeat,
            } => {
                if *symno != 0 {
                    if *expression == 0 && (*repeat == 0 || *repeat == 1) {
                        if *symno > 0 {
                            write!(f, "\t{}\tSYM_{}", ".word", *symno)
                        } else {
                            write!(f, "\t{}\t${}", ".word", -*symno)
                        }
                    } else {
                        eprintln!("{:?}", self);
                        unimplemented!()
                    }
                } else {
                    write!(f, "\t.word\t{expression} : {repeat}")
                }
            }
            Asm::igpword {
                symno,
                expression: _,
                repeat: _,
            } => {
                if *symno > 0 {
                    write!(f, "\t{}\tSYM_{}", ".gpword", *symno)
                } else {
                    write!(f, "\t{}\t${}", ".gpword", -*symno)
                }
            }
            Asm::imask { regmask, regoffset } => write!(f, "\t.mask\t0x{regmask:08X}, {regoffset}"),
            Asm::ifmask { regmask, regoffset } => {
                write!(f, "\t.fmask\t0x{regmask:08X}, {regoffset}")
            }
            Asm::icpload { symno: _, args } => {
                if let ArgFormat::frob {
                    symno: _,
                    reg,
                    offset: _,
                    base: _,
                    mem_tag: _,
                } = args
                {
                    write!(f, "\t.cpload\t{}", reg)
                } else {
                    unimplemented!()
                }
            }
            Asm::icprestore { offset } => {
                write!(f, "\t.cprestore\t{offset}")
            }
            Asm::icpadd { symno: _, args } => {
                if let ArgFormat::frob {
                    symno: _,
                    reg,
                    offset: _,
                    base: _,
                    mem_tag: _,
                } = args
                {
                    write!(f, "\t.cpadd\t{reg}")
                } else {
                    unimplemented!()
                }
            }
            Asm::ilcomm { symno, length } => {
                if *symno > 0 {
                    write!(f, "\t.lcomm\tSYM_{} {length}", *symno)
                } else {
                    write!(f, "\t.lcomm\t${} {length}", -*symno)
                }
            }
            Asm::ioption { opt, value } => {
                write!(f, "\t.option\t{opt}{value}")
            }
            Asm::iocode { op, args } => {
                write!(f, "\t{op}\t")?;
                match args {
                    ArgFormat::frob {
                        symno,
                        reg,
                        offset,
                        base,
                        mem_tag: _,
                    } => {
                        if *symno != 0 {
                            if *offset == 0 {
                                if *symno > 0 {
                                    write!(f, "{reg}, SYM_{symno}({base})")
                                } else {
                                    write!(f, "{reg}, ${}({base})", -*symno)
                                }
                            } else {
                                println!("{:?}", self);
                                unimplemented!()
                            }
                        } else {
                            write!(f, "{reg}, {offset}({base})")
                        }
                    }
                    ArgFormat::fri {
                        reg,
                        mem_tag: _,
                        immediate,
                    } => {
                        write!(f, "{reg}, {immediate}")
                    }
                    ArgFormat::fa {
                        symno,
                        base,
                        mem_tag: _,
                        immediate,
                    } => {
                        if *base == register::xnoreg {
                            if *symno == 0 {
                                write!(f, "{immediate}")
                            } else {
                                if *symno > 0 {
                                    write!(f, " SYM_{}", *symno)?;
                                } else {
                                    write!(f, " ${}", -*symno)?;
                                }
                                if *immediate != 0 {
                                    println!("{:?}", self);
                                    unimplemented!()
                                } else {
                                    write!(f, "")
                                }
                            }
                        } else {
                            println!("{:?}", self);
                            unimplemented!()
                        }
                    }
                    ArgFormat::fr { reg } => {
                        write!(f, "{reg}")
                    }
                    ArgFormat::fl { symno } => {
                        // TODO: read symtab
                        if *symno > 0 {
                            write!(f, "SYM_{}", *symno)
                        } else {
                            write!(f, "${}", -*symno)
                        }
                    }
                    ArgFormat::fril {
                        symno,
                        reg,
                        immediate,
                        mem_tag: _,
                    } => {
                        if *symno > 0 {
                            write!(f, "{reg}, {immediate}, SYM_{}", *symno)
                        } else {
                            write!(f, "{reg}, {immediate}, ${}", -*symno)
                        }
                    }
                    ArgFormat::frr { reg1, reg2 } => {
                        write!(f, "{reg1}, {reg2}")
                    }
                    ArgFormat::frri {
                        reg1,
                        reg2,
                        immediate,
                        mem_tag: _,
                    } => {
                        write!(f, "{reg1}, {reg2}, {immediate}")
                    }
                    ArgFormat::fra { symno, reg, offset } => {
                        // TODO: read from symtab
                        if *symno > 0 {
                            write!(f, "{reg}, SYM_{}", *symno)
                        } else {
                            write!(f, "{reg},  ${}", -*symno)
                        }?;
                        if *offset != 0 {
                            if *offset > 0 {
                                write!(f, "+{}", *offset)
                            } else {
                                write!(f, "-{}", -*offset)
                            }
                        } else {
                            write!(f, "")
                        }
                    }

                    _ => write!(f, "{self:?}"),
                }
            }
            _ => write!(f, "{self:?}"),
        }
    }
}

fn get_bits(word: u32, offset: u32, count: u32) -> u32 {
    (word >> (0x20 - offset - count)) & ((1 << count) - 1)
}

fn process_args(bytes: &[u8], off: &mut usize) -> Option<ArgFormat> {
    let record = &bytes[*off..*off + BINASM_RECORD_LENGTH];
    let uwords: Vec<u32> = record
        .chunks_exact(4)
        .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
        .collect();
    let iwords: Vec<i32> = record
        .chunks_exact(4)
        .map(|b| i32::from_be_bytes(b.try_into().unwrap()))
        .collect();
    let form_idx = get_bits(uwords[2], 14, 4);
    let form = format::from_repr(form_idx as usize).unwrap();

    let symno = iwords[0];
    let reg1_idx = get_bits(uwords[2], 0, 7);
    let reg1 = register::from_repr(reg1_idx as usize).unwrap();
    let reg2_idx = get_bits(uwords[2], 7, 7);
    let reg2 = register::from_repr(reg2_idx as usize).unwrap();
    let mem_tag = get_bits(uwords[2], 18, 14);

    match form {
        format::frob => {
            let reg = reg1;
            let base = reg2;
            let offset = iwords[3] as i16;
            Some(ArgFormat::frob {
                symno,
                reg,
                offset,
                base,
                mem_tag,
            })
        }
        format::fra => {
            let reg = reg1;
            let offset = iwords[3];
            // println!("{:?}", record);
            Some(ArgFormat::fra { symno, reg, offset })
        }
        format::fri => {
            let reg = reg1;
            let mem_tag = get_bits(uwords[2], 18, 14);
            let immediate = iwords[3];

            Some(ArgFormat::fri {
                reg,
                mem_tag,
                immediate,
            })
        }
        format::frrr => {
            let reg1 = reg1;
            let reg2 = reg2;
            let reg3_idx = get_bits(uwords[2], 14, 7);
            let reg3 = register::from_repr(reg3_idx as usize).unwrap();

            Some(ArgFormat::frrr { reg1, reg2, reg3 })
        }
        format::frri => {
            let reg1 = reg1;
            let reg2 = reg2;
            let immediate = iwords[3];
            let mem_tag = get_bits(uwords[2], 18, 14);

            Some(ArgFormat::frri {
                reg1,
                reg2,
                immediate,
                mem_tag,
            })
        }
        format::frr => Some(ArgFormat::frr { reg1, reg2 }),
        format::fa => {
            // println!("{:?}", record);
            let base = reg1;
            let mem_tag = get_bits(uwords[2], 18, 14);
            let immediate = iwords[3];

            Some(ArgFormat::fa {
                symno,
                base,
                mem_tag,
                immediate,
            })
        }
        format::fr => {
            let reg = reg1;

            Some(ArgFormat::fr { reg })
        }
        format::frrl => Some(ArgFormat::frrl { symno, reg1, reg2 }),
        format::frl => {
            let reg = reg1;
            Some(ArgFormat::frl { symno, reg })
        }
        format::fl => Some(ArgFormat::fl { symno }),
        format::forrr => Some(ArgFormat::forrr),
        format::fril => {
            let reg = reg1;
            let immediate = iwords[3];

            Some(ArgFormat::fril {
                reg,
                immediate,
                symno,
                mem_tag,
            })
        }
        format::fi => {
            let immediate = iwords[3];

            Some(ArgFormat::fi { immediate })
        }
        format::foa => Some(ArgFormat::foa),
        format::frrrr => {
            let reg1 = reg1;
            let reg2 = reg2;
            let reg3_idx = get_bits(uwords[2], 14, 7);
            let reg3 = register::from_repr(reg3_idx as usize).unwrap();
            let reg4_idx = get_bits(uwords[2], 21, 7);
            let reg4 = register::from_repr(reg4_idx as usize).unwrap();

            Some(ArgFormat::frrrr {
                reg1,
                reg2,
                reg3,
                reg4,
            })
        }
    }
}

fn process_iocode(bytes: &[u8], off: &mut usize) -> Option<Asm> {
    let asm; // = None;
    let record = &bytes[*off..*off + BINASM_RECORD_LENGTH];
    let uwords: Vec<u32> = record
        .chunks_exact(4)
        .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
        .collect();
    // let iwords: Vec<i32> = record.chunks_exact(4).map(|b| i32::from_be_bytes(b.try_into().unwrap())).collect();
    let op_idx = get_bits(uwords[1], 31 - 9, 9);
    let op = asmcode::from_repr(op_idx as usize).unwrap();
    let args = process_args(bytes, off).unwrap();

    asm = Some(Asm::iocode { op, args });

    asm
}

#[derive(Debug)]
enum OptValue {
    None,
    Int(i32),
    Float(f32),
    String(String),
}

impl fmt::Display for OptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptValue::None => write!(f, ""),
            OptValue::Int(v) => write!(f, "{v}"),
            OptValue::Float(v) => write!(f, "{v}"),
            OptValue::String(v) => write!(f, "{v}"),
        }
    }
}

fn process_ioption(bytes: &[u8], off: &mut usize) -> Option<Asm> {
    let asm; // = None;
    let record = &bytes[*off..*off + BINASM_RECORD_LENGTH];
    let uwords: Vec<u32> = record
        .chunks_exact(mem::size_of::<u32>())
        .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
        .collect();
    let iwords: Vec<i32> = record
        .chunks_exact(4)
        .map(|b| i32::from_be_bytes(b.try_into().unwrap()))
        .collect();

    let option_idx = get_bits(uwords[1], 16, 2);
    let option_type = opt_type::from_repr(option_idx as usize).unwrap();

    let option_arg_type = match option_type {
        opt_type::o_undefined => opt_arg_type::opt_none,
        opt_type::o_optimize => opt_arg_type::opt_int,
        opt_type::o_pic => opt_arg_type::opt_int,
    };

    match option_arg_type {
        opt_arg_type::opt_none => {
            let value = OptValue::None;

            asm = Some(Asm::ioption {
                opt: option_type,
                value,
            })
        }
        opt_arg_type::opt_int => {
            let value = OptValue::Int(iwords[3]);

            asm = Some(Asm::ioption {
                opt: option_type,
                value,
            })
        }
        // unimplemented since unusable, as1 does not read any others
        opt_arg_type::opt_float | opt_arg_type::opt_string => unimplemented!(),
    }

    asm
}

fn process_record(bytes: &[u8], off: &mut usize) -> Option<Asm> {
    let mut asm = None;
    let record = &bytes[*off..*off + BINASM_RECORD_LENGTH];
    let uwords: Vec<u32> = record
        .chunks_exact(mem::size_of::<u32>())
        .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
        .collect();
    let iwords: Vec<i32> = record
        .chunks_exact(4)
        .map(|b| i32::from_be_bytes(b.try_into().unwrap()))
        .collect();

    let t = get_bits(uwords[1], 10, 6) as usize;
    let itype = Itype::from_repr(t as usize).unwrap();
    // println!("{t:?} -> {itype:?}");
    let symno = iwords[0];
    let lexlev = iwords[2];
    let length = uwords[2];
    let repeat = uwords[3];

    match itype {
        Itype::ierr => asm = Some(Asm::ierr),
        Itype::idata => asm = Some(Asm::idata { symno }),
        Itype::iend => asm = Some(Asm::iend { symno }),
        Itype::iglobal => asm = Some(Asm::iglobal { symno }),
        Itype::iasm0 => asm = Some(Asm::iasm0 { symno }),
        Itype::iendrep => asm = Some(Asm::iendrep { symno }),
        Itype::ilabel => asm = Some(Asm::ilabel { symno }),
        Itype::ialloc => asm = Some(Asm::ialloc { symno }),
        Itype::isdata => asm = Some(Asm::isdata { symno }),
        Itype::irdata => asm = Some(Asm::irdata { symno }),
        Itype::ilab => asm = Some(Asm::ilab { symno }),
        Itype::ibgnb => asm = Some(Asm::ibgnb { symno }),
        Itype::irestext => asm = Some(Asm::irestext { symno }),
        Itype::iendb => asm = Some(Asm::iendb { symno }),

        Itype::ient => asm = Some(Asm::ient { symno, lexlev }),
        Itype::iaent => asm = Some(Asm::iaent { symno, lexlev }),
        Itype::iprologue => asm = Some(Asm::iprologue { symno, lexlev }),
        Itype::iweakext => asm = Some(Asm::iweakext { symno, lexlev }),

        Itype::iframe => {
            let frameoffset = i32::from_be_bytes(record[8..0xC].try_into().unwrap());
            let word3 = u32::from_be_bytes(record[0xC..0x10].try_into().unwrap());
            let framereg = register::from_repr(get_bits(word3, 0, 7) as usize).unwrap();
            let pcreg = register::from_repr(get_bits(word3, 7, 7) as usize).unwrap();

            asm = Some(Asm::iframe {
                frameoffset,
                framereg,
                pcreg,
            });
        }
        Itype::imask | Itype::ifmask => {
            let regmask = uwords[2];
            let regoffset = iwords[3];

            asm = Some(match itype {
                Itype::imask => Asm::imask { regmask, regoffset },
                Itype::ifmask => Asm::ifmask { regmask, regoffset },
                _ => unreachable!(),
            });
            // println!("{:?}", asm.as_ref().unwrap());
        }
        Itype::iverstamp => {
            let majornumber = iwords[2];
            let minornumber = iwords[3];

            asm = Some(Asm::iverstamp {
                majornumber,
                minornumber,
            });
        }
        Itype::iloc => {
            let filenumber = uwords[2];
            let linenumber = uwords[3];

            asm = Some(Asm::iloc {
                filenumber,
                linenumber,
            });
        }

        Itype::ialign => {
            asm = Some(Asm::ialign { length });
        }
        Itype::iset => {
            let value = set_value::from_repr(length as usize).unwrap();
            asm = Some(Asm::iset { value });
        }
        Itype::iascii | Itype::iasciiz | Itype::ifile => {
            let mut string = AsciiString { s: Vec::new() };
            bytes[*off + BINASM_RECORD_LENGTH..*off + BINASM_RECORD_LENGTH + length as usize]
                .clone_into(&mut string.s);
            asm = Some(match itype {
                Itype::iascii => Asm::iascii {
                    length,
                    repeat,
                    string,
                },
                Itype::iasciiz => Asm::iasciiz {
                    length,
                    repeat,
                    string,
                },
                Itype::ifile => Asm::ifile {
                    symno,
                    length,
                    repeat,
                    string,
                },
                _ => unreachable!(),
            });

            let extra = (length as usize + BINASM_RECORD_LENGTH - 1) / BINASM_RECORD_LENGTH
                * BINASM_RECORD_LENGTH;
            *off += extra;
        }
        Itype::icomm | Itype::ilcomm => {
            asm = match itype {
                Itype::icomm => Some(Asm::icomm { symno, length }),
                Itype::ilcomm => Some(Asm::ilcomm { symno, length }),
                _ => unreachable!(),
            }
        }
        Itype::ifloat | Itype::idouble | Itype::iextended => {
            let repeat = uwords[3];
            let mut string = FPString { s: Vec::new() };
            bytes[*off + BINASM_RECORD_LENGTH..*off + BINASM_RECORD_LENGTH + length as usize]
                .clone_into(&mut string.s);

            asm = match itype {
                Itype::ifloat => Some(Asm::ifloat {
                    length,
                    repeat,
                    string,
                }),
                Itype::idouble => Some(Asm::idouble {
                    length,
                    repeat,
                    string,
                }),
                Itype::iextended => None,
                _ => unreachable!(),
            };
            let extra = (length as usize + BINASM_RECORD_LENGTH - 1) / BINASM_RECORD_LENGTH
                * BINASM_RECORD_LENGTH;
            *off += extra;
        }
        Itype::itext => {
            asm = Some(Asm::itext);
        }
        Itype::icprestore => {
            let offset = uwords[2];

            asm = Some(Asm::icprestore { offset })
        }
        Itype::ialign
        | Itype::iascii
        | Itype::iasciiz
        | Itype::icomm
        | Itype::ilcomm
        | Itype::isym
        | Itype::ifloat
        | Itype::idouble
        | Itype::iextended
        | Itype::irep
        | Itype::iset
        | Itype::ispace
        | Itype::ifile
        | Itype::iglobabs
        | Itype::iextern
        | Itype::ishift_addr
        | Itype::itext
        | Itype::icprestore => match itype {
            _ => {}
        },

        Itype::ilivereg => {
            let gpmask = uwords[2];
            let fpmask = uwords[3];
            asm = Some(Asm::ilivereg { gpmask, fpmask });
        }
        Itype::idword => {
            let expression = iwords[2];

            asm = Some(Asm::idword { expression, repeat })
        }
        Itype::ibyte => {
            let expression = iwords[2];

            asm = Some(Asm::ibyte {
                symno,
                expression,
                repeat,
            })
        }
        Itype::ihalf => {
            let expression = iwords[2];

            asm = Some(Asm::ihalf {
                symno,
                expression,
                repeat,
            })
        }
        Itype::iword => {
            let expression = iwords[2];

            asm = Some(Asm::iword {
                symno,
                expression,
                repeat,
            })
        }
        Itype::igpword => {
            let expression = iwords[2];

            // println!("{:?}", record);
            asm = Some(Asm::igpword {
                symno,
                expression,
                repeat,
            })
        }

        Itype::iocode => {
            asm = process_iocode(bytes, off);
        }
        Itype::icpload => {
            let args = process_args(bytes, off).unwrap();
            asm = Some(Asm::icpload { symno, args });
        }
        Itype::icpadd => {
            let args = process_args(bytes, off).unwrap();
            asm = Some(Asm::icpadd { symno, args });
        }
        Itype::ivreg | Itype::icpload | Itype::icpalias | Itype::icpadd => {
            // print!("{:?} ", itype);
            // asm = process_iocode(bytes, off);
        }
        Itype::ioption => asm = process_ioption(bytes, off),
        _ => {}
    }

    if asm.is_some() {
        println!("{}", asm.as_ref().unwrap());
        return asm;
    }
    print!("{:12}: ", format!("{itype:?}"));
    for b in record {
        print!("{b:02X} ");
    }
    println!();
    None
}

// Returns number of bytes read
pub fn process_records(bytes: &[u8]) -> usize {
    let mut off = 0;
    while off < bytes.len() {
        process_record(bytes, &mut off);
        off += BINASM_RECORD_LENGTH;
    }
    // let mut is_data = false;
    // for record in bytes.chunks_exact(BINASM_RECORD_LENGTH) {
    //     let t = (record[5] & 0b11111) as usize;
    //     let itype = Itype::from_repr(t).unwrap();

    //     match itype {
    //         _ => {
    //             print!("{:12}: ", format!("{itype:?}"));
    //             for b in record {
    //                 print!("{b:02X} ");
    //             }
    //             println!()
    //         }
    //     }

    //     is_data = false;
    // }
    // let mut off = 0;

    // while off < b.len() {

    // }

    return off;
}
