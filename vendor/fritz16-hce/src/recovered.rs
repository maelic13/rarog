//! Mechanically recovered basic blocks; regenerate with tools/lift_hce.py.

use super::{Flags, Memory};

// Source SHA-256: 264f2eede4c737f938c726e3c91e5ce07460e998110cf197abfc463dfa4622ae
pub(super) fn eval_14009e270(
    m: &mut Memory,
    registers: &mut [u64; 16],
    simd: &mut [u128; 2],
) -> u64 {
    let mut r = *registers;
    let mut x = *simd;
    let mut f = Flags::default();
    let mut pc = 0x14009e270u64;
    loop {
        match pc {
            0x14009e270 => {
                // 14009e270  48895c2410  MOV qword ptr [RSP + 0x10],RBX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x10u64), r[3]);
                }
                // 14009e275  48896c2418  MOV qword ptr [RSP + 0x18],RBP
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x18u64), r[5]);
                }
                // 14009e27a  4889742420  MOV qword ptr [RSP + 0x20],RSI
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[6]);
                }
                // 14009e27f  57  PUSH RDI
                {
                    r[4] = r[4].wrapping_sub(8);
                    m.write::<64>(r[4], r[7]);
                }
                // 14009e280  4154  PUSH R12
                {
                    r[4] = r[4].wrapping_sub(8);
                    m.write::<64>(r[4], r[12]);
                }
                // 14009e282  4155  PUSH R13
                {
                    r[4] = r[4].wrapping_sub(8);
                    m.write::<64>(r[4], r[13]);
                }
                // 14009e284  4156  PUSH R14
                {
                    r[4] = r[4].wrapping_sub(8);
                    m.write::<64>(r[4], r[14]);
                }
                // 14009e286  4157  PUSH R15
                {
                    r[4] = r[4].wrapping_sub(8);
                    m.write::<64>(r[4], r[15]);
                }
                // 14009e288  4881eca0000000  SUB RSP,0xa0
                {
                    let v = f.arithmetic(r[4], 0xa0u64, 64, true);
                    r[4] = v;
                }
                // 14009e28f  488b051a2f3200  MOV RAX,qword ptr [0x1403c11b0]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c11b0u64));
                }
                // 14009e296  48c1e807  SHR RAX,0x7
                {
                    let v = r[0];
                    let n = 7u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009e29a  25fe3f0000  AND EAX,0x3ffe
                {
                    let v = (r[0] & 0xffffffffu64 & 0x3ffeu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009e29f  48894c2478  MOV qword ptr [RSP + 0x78],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x78u64), r[1]);
                }
                // 14009e2a4  48c1e006  SHL RAX,0x6
                {
                    let v = r[0];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009e2a8  48030571722b00  ADD RAX,qword ptr [0x140355520]
                {
                    let v = f.arithmetic(
                        r[0],
                        m.read::<64>(0u64.wrapping_add(0x140355520u64)),
                        64,
                        false,
                    );
                    r[0] = v;
                }
                // 14009e2af  0f1800  NOP dword ptr [RAX]
                // 14009e2b2  4c8b05672e3200  MOV R8,qword ptr [0x1403c1120]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1120u64));
                }
                // 14009e2b9  48b97f7f7f7f7f7fffff  MOV RCX,-0x808080808081
                {
                    r[1] = 0xffff7f7f7f7f7f7fu64;
                }
                // 14009e2c3  488b05462e3200  MOV RAX,qword ptr [0x1403c1110]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 14009e2ca  bb20000000  MOV EBX,0x20
                {
                    r[3] = (0x20u64) & 0xffff_ffff;
                }
                // 14009e2cf  488b15522e3200  MOV RDX,qword ptr [0x1403c1128]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1128u64));
                }
                // 14009e2d6  4c8bf8  MOV R15,RAX
                {
                    r[15] = r[0];
                }
                // 14009e2d9  488b3d202e3200  MOV RDI,qword ptr [0x1403c1100]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(0x1403c1100u64));
                }
                // 14009e2e0  4c23f9  AND R15,RCX
                {
                    let v = r[15] & r[1];
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009e2e3  49c1e702  SHL R15,0x2
                {
                    let v = r[15];
                    let n = 2u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[15] = v;
                    }
                }
                // 14009e2e7  48b9fefefefefefefe00  MOV RCX,0xfefefefefefefe
                {
                    r[1] = 0xfefefefefefefeu64;
                }
                // 14009e2f1  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e2f4  895c2430  MOV dword ptr [RSP + 0x30],EBX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x30u64),
                        r[3] & 0xffffffffu64,
                    );
                }
                // 14009e2f8  4c0bf8  OR R15,RAX
                {
                    let v = r[15] | r[0];
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009e2fb  895c2440  MOV dword ptr [RSP + 0x40],EBX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x40u64),
                        r[3] & 0xffffffffu64,
                    );
                }
                // 14009e2ff  488b05122e3200  MOV RAX,qword ptr [0x1403c1118]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 14009e306  48b980bfbfbfbfbfbf3f  MOV RCX,0x3fbfbfbfbfbfbf80
                {
                    r[1] = 0x3fbfbfbfbfbfbf80u64;
                }
                // 14009e310  4c8bf0  MOV R14,RAX
                {
                    r[14] = r[0];
                }
                // 14009e313  49c1e707  SHL R15,0x7
                {
                    let v = r[15];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[15] = v;
                    }
                }
                // 14009e317  49c1ee02  SHR R14,0x2
                {
                    let v = r[14];
                    let n = 2u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[14] = v;
                    }
                }
                // 14009e31b  4d8bc8  MOV R9,R8
                {
                    r[9] = r[8];
                }
                // 14009e31e  4c0b0d0b2e3200  OR R9,qword ptr [0x1403c1130]
                {
                    let v = r[9] | m.read::<64>(0u64.wrapping_add(0x1403c1130u64));
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009e325  4c23f1  AND R14,RCX
                {
                    let v = r[14] & r[1];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 14009e328  4c0b0d112e3200  OR R9,qword ptr [0x1403c1140]
                {
                    let v = r[9] | m.read::<64>(0u64.wrapping_add(0x1403c1140u64));
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009e32f  48b9007f7f7f7f7f7f7f  MOV RCX,0x7f7f7f7f7f7f7f00
                {
                    r[1] = 0x7f7f7f7f7f7f7f00u64;
                }
                // 14009e339  4c0b0d102e3200  OR R9,qword ptr [0x1403c1150]
                {
                    let v = r[9] | m.read::<64>(0u64.wrapping_add(0x1403c1150u64));
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009e340  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e343  488b0dbe2d3200  MOV RCX,qword ptr [0x1403c1108]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                }
                // 14009e34a  4c0bf0  OR R14,RAX
                {
                    let v = r[14] | r[0];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 14009e34d  49c1ee07  SHR R14,0x7
                {
                    let v = r[14];
                    let n = 7u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[14] = v;
                    }
                }
                // 14009e351  490bcf  OR RCX,R15
                {
                    let v = r[1] | r[15];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009e354  490bfe  OR RDI,R14
                {
                    let v = r[7] | r[14];
                    f.logic(v, 64);
                    r[7] = v;
                }
                // 14009e357  4c897c2428  MOV qword ptr [RSP + 0x28],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[15]);
                }
                // 14009e35c  4d23ce  AND R9,R14
                {
                    let v = r[9] & r[14];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009e35f  4c89742438  MOV qword ptr [RSP + 0x38],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64), r[14]);
                }
                // 14009e364  48f7d7  NOT RDI
                {
                    let v = !(r[7]);
                    r[7] = v;
                }
                // 14009e367  4c894c2448  MOV qword ptr [RSP + 0x48],R9
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64), r[9]);
                }
                // 14009e36c  4c8d0d8d1cf6ff  LEA R9,[0x140000000]
                {
                    r[9] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009e373  4c897c2470  MOV qword ptr [RSP + 0x70],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64), r[15]);
                }
                // 14009e378  48f7d1  NOT RCX
                {
                    let v = !(r[1]);
                    r[1] = v;
                }
                // 14009e37b  4c89742468  MOV qword ptr [RSP + 0x68],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x68u64), r[14]);
                }
                // 14009e380  480bfa  OR RDI,RDX
                {
                    let v = r[7] | r[2];
                    f.logic(v, 64);
                    r[7] = v;
                }
                // 14009e383  490bc8  OR RCX,R8
                {
                    let v = r[1] | r[8];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009e386  480b3dab2d3200  OR RDI,qword ptr [0x1403c1138]
                {
                    let v = r[7] | m.read::<64>(0u64.wrapping_add(0x1403c1138u64));
                    f.logic(v, 64);
                    r[7] = v;
                }
                // 14009e38d  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 14009e390  480b05a12d3200  OR RAX,qword ptr [0x1403c1138]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1138u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e397  4d8be6  MOV R12,R14
                {
                    r[12] = r[14];
                }
                // 14009e39a  480b05a72d3200  OR RAX,qword ptr [0x1403c1148]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e3a1  8beb  MOV EBP,EBX
                {
                    r[5] = (r[3] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e3a3  480b05ae2d3200  OR RAX,qword ptr [0x1403c1158]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1158u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e3aa  480b0d7f2d3200  OR RCX,qword ptr [0x1403c1130]
                {
                    let v = r[1] | m.read::<64>(0u64.wrapping_add(0x1403c1130u64));
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009e3b1  4923c7  AND RAX,R15
                {
                    let v = r[0] & r[15];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e3b4  4889442458  MOV qword ptr [RSP + 0x58],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64), r[0]);
                }
                // 14009e3b9  486305b02d3200  MOVSXD RAX,dword ptr [0x1403c1170]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1170u64))) as i32 as i64) as u64;
                }
                // 14009e3c0  4889bc2490000000  MOV qword ptr [RSP + 0x90],RDI
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x90u64), r[7]);
                }
                // 14009e3c8  48898c2480000000  MOV qword ptr [RSP + 0x80],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x80u64), r[1]);
                }
                // 14009e3d0  498b84c1f0f40e00  MOV RAX,qword ptr [R9 + RAX*0x8 + 0xef4f0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xef4f0u64),
                    );
                }
                // 14009e3d8  4889442450  MOV qword ptr [RSP + 0x50],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64), r[0]);
                }
                // 14009e3dd  486305902d3200  MOVSXD RAX,dword ptr [0x1403c1174]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 14009e3e4  498bb4c1f0f40e00  MOV RSI,qword ptr [R9 + RAX*0x8 + 0xef4f0]
                {
                    r[6] = m.read::<64>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xef4f0u64),
                    );
                }
                // 14009e3ec  4889742460  MOV qword ptr [RSP + 0x60],RSI
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64), r[6]);
                }
                // 14009e3f1  4d85c0  TEST R8,R8
                {
                    let v = r[8];
                    f.logic(v, 64);
                }
                // 14009e3f4  7420  JZ 0x14009e416
                pc = if f.z { 0x14009e416 } else { 0x14009e3f6 };
                continue;
            }
            0x14009e3f6 => {
                // 14009e3f6  490fbcc0  BSF RAX,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009e3fa  8905180f2600  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009e400  4d0bbcc1f0fa0e00  OR R15,qword ptr [R9 + RAX*0x8 + 0xefaf0]
                {
                    let v = r[15]
                        | m.read::<64>(
                            0u64.wrapping_add(r[9])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xefaf0u64),
                        );
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009e408  498d40ff  LEA RAX,[R8 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[8]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009e40c  4c23c0  AND R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009e40f  75e5  JNZ 0x14009e3f6
                pc = if !f.z { 0x14009e3f6 } else { 0x14009e411 };
                continue;
            }
            0x14009e411 => {
                // 14009e411  4c897c2428  MOV qword ptr [RSP + 0x28],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[15]);
                }
                pc = 0x14009e416;
            }
            0x14009e416 => {
                // 14009e416  4885d2  TEST RDX,RDX
                {
                    let v = r[2];
                    f.logic(v, 64);
                }
                // 14009e419  7425  JZ 0x14009e440
                pc = if f.z { 0x14009e440 } else { 0x14009e41b };
                continue;
            }
            0x14009e41b => {
                // 14009e41b  0f1f440000  NOP dword ptr [RAX + RAX*0x1]
                pc = 0x14009e420;
            }
            0x14009e420 => {
                // 14009e420  480fbcc2  BSF RAX,RDX
                {
                    let v = r[2];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009e424  8905ee0e2600  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009e42a  4d0bb4c1f0fa0e00  OR R14,qword ptr [R9 + RAX*0x8 + 0xefaf0]
                {
                    let v = r[14]
                        | m.read::<64>(
                            0u64.wrapping_add(r[9])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xefaf0u64),
                        );
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 14009e432  488d42ff  LEA RAX,[RDX + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009e436  4823d0  AND RDX,RAX
                {
                    let v = r[2] & r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 14009e439  75e5  JNZ 0x14009e420
                pc = if !f.z { 0x14009e420 } else { 0x14009e43b };
                continue;
            }
            0x14009e43b => {
                // 14009e43b  4c89742438  MOV qword ptr [RSP + 0x38],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64), r[14]);
                }
                pc = 0x14009e440;
            }
            0x14009e440 => {
                // 14009e440  e8abbc0200  CALL 0x1400ca0f0
                {
                    r[4] = r[4].wrapping_sub(8);
                    r[0] = pawn_1400ca0f0(m, &mut r, &mut x);
                    r[4] = r[4].wrapping_add(8);
                }
                // 14009e445  4c8b15e42c3200  MOV R10,qword ptr [0x1403c1130]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1130u64));
                }
                // 14009e44c  4c8be8  MOV R13,RAX
                {
                    r[13] = r[0];
                }
                // 14009e44f  4889842488000000  MOV qword ptr [RSP + 0x88],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x88u64), r[0]);
                }
                // 14009e457  488b442478  MOV RAX,qword ptr [RSP + 0x78]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x78u64));
                }
                // 14009e45c  6950742c001600  IMUL EDX,dword ptr [RAX + 0x74],0x16002c
                {
                    r[2] = ((m.read::<32>(0u64.wrapping_add(r[0]).wrapping_add(0x74u64)))
                        .wrapping_mul(0x16002cu64))
                        & 0xffff_ffff;
                }
                // 14009e463  488b050e2d3200  MOV RAX,qword ptr [0x1403c1178]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                }
                // 14009e46a  41035530  ADD EDX,dword ptr [R13 + 0x30]
                {
                    let v = f.arithmetic(
                        r[2] & 0xffffffffu64,
                        m.read::<32>(0u64.wrapping_add(r[13]).wrapping_add(0x30u64)),
                        32,
                        false,
                    );
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009e46e  0315242d3200  ADD EDX,dword ptr [0x1403c1198]
                {
                    let v = f.arithmetic(
                        r[2] & 0xffffffffu64,
                        m.read::<32>(0u64.wrapping_add(0x1403c1198u64)),
                        32,
                        false,
                    );
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009e474  4d85d2  TEST R10,R10
                {
                    let v = r[10];
                    f.logic(v, 64);
                }
                // 14009e477  0f84c2000000  JZ 0x14009e53f
                pc = if f.z { 0x14009e53f } else { 0x14009e47d };
                continue;
            }
            0x14009e47d => {
                // 14009e47d  4c8b1dcc2c3200  MOV R11,qword ptr [0x1403c1150]
                {
                    r[11] = m.read::<64>(0u64.wrapping_add(0x1403c1150u64));
                }
                // 14009e484  488d2d751bf6ff  LEA RBP,[0x140000000]
                {
                    r[5] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009e48b  4c33d8  XOR R11,RAX
                {
                    let v = r[11] ^ r[0];
                    f.logic(v, 64);
                    r[11] = v;
                }
                // 14009e48e  6690  NOP
                pc = 0x14009e490;
            }
            0x14009e490 => {
                // 14009e490  490fbcc2  BSF RAX,R10
                {
                    let v = r[10];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009e494  4c63c8  MOVSXD R9,EAX
                {
                    r[9] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009e497  89057b0e2600  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009e49d  4a8b84cdf0040f00  MOV RAX,qword ptr [RBP + R9*0x8 + 0xf04f0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[5])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0xf04f0u64),
                    );
                }
                // 14009e4a5  428b8c8d00090f00  MOV ECX,dword ptr [RBP + R9*0x4 + 0xf0900]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[5])
                            .wrapping_add(r[9].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0900u64),
                    )) & 0xffff_ffff;
                }
                // 14009e4ad  4e8b84cdf0e00e00  MOV R8,qword ptr [RBP + R9*0x8 + 0xee0f0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[5])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0xee0f0u64),
                    );
                }
                // 14009e4b5  4d23c3  AND R8,R11
                {
                    let v = r[8] & r[11];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009e4b8  4e0faf84cdf0000f00  IMUL R8,qword ptr [RBP + R9*0x8 + 0xf00f0]
                {
                    r[8] = (r[8]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[5])
                                .wrapping_add(r[9].wrapping_mul(0x8u64))
                                .wrapping_add(0xf00f0u64),
                        ),
                    );
                }
                // 14009e4c1  49d3e8  SHR R8,CL
                {
                    let v = r[8];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 14009e4c4  4a8b0cc0  MOV RCX,qword ptr [RAX + R8*0x8]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[8].wrapping_mul(0x8u64)),
                    );
                }
                // 14009e4c8  8d4311  LEA EAX,[RBX + 0x11]
                {
                    r[0] = (0u64.wrapping_add(r[3]).wrapping_add(0x11u64)) & 0xffff_ffff;
                }
                // 14009e4cb  4c0bf9  OR R15,RCX
                {
                    let v = r[15] | r[1];
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009e4ce  4885ce  TEST RSI,RCX
                {
                    let v = r[6] & r[1];
                    f.logic(v, 64);
                }
                // 14009e4d1  0f44c3  CMOVZ EAX,EBX
                {
                    if f.z {
                        r[0] = (r[3] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e4d4  4823cf  AND RCX,RDI
                {
                    let v = r[1] & r[7];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009e4d7  8bd8  MOV EBX,EAX
                {
                    r[3] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e4d9  f3480fb8c1  POPCNT RAX,RCX
                {
                    let v = r[1];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009e4de  4469c01b001b00  IMUL R8D,EAX,0x1b001b
                {
                    r[8] = ((r[0] & 0xffffffffu64).wrapping_mul(0x1b001bu64)) & 0xffff_ffff;
                }
                // 14009e4e5  4a8b84cde0641000  MOV RAX,qword ptr [RBP + R9*0x8 + 0x1064e0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[5])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0x1064e0u64),
                    );
                }
                // 14009e4ed  480fafc1  IMUL RAX,RCX
                {
                    r[0] = (r[0]).wrapping_mul(r[1]);
                }
                // 14009e4f1  49c1e104  SHL R9,0x4
                {
                    let v = r[9];
                    let n = 4u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 14009e4f5  48c1e83c  SHR RAX,0x3c
                {
                    let v = r[0];
                    let n = 60u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009e4f9  4903c1  ADD RAX,R9
                {
                    let v = f.arithmetic(r[0], r[9], 64, false);
                    r[0] = v;
                }
                // 14009e4fc  44038485e0661000  ADD R8D,dword ptr [RBP + RAX*0x4 + 0x1066e0]
                {
                    let v = f.arithmetic(
                        r[8] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[5])
                                .wrapping_add(r[0].wrapping_mul(0x4u64))
                                .wrapping_add(0x1066e0u64),
                        ),
                        32,
                        false,
                    );
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009e504  498bc2  MOV RAX,R10
                {
                    r[0] = r[10];
                }
                // 14009e507  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 14009e50a  49234528  AND RAX,qword ptr [R13 + 0x28]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x28u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e50e  4985c2  TEST R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                }
                // 14009e511  8d821e002d00  LEA EAX,[RDX + 0x2d001e]
                {
                    r[0] = (0u64.wrapping_add(r[2]).wrapping_add(0x2d001eu64)) & 0xffff_ffff;
                }
                // 14009e517  0f45c2  CMOVNZ EAX,EDX
                {
                    if !f.z {
                        r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e51a  428d1400  LEA EDX,[RAX + R8*0x1]
                {
                    r[2] = (0u64
                        .wrapping_add(r[0])
                        .wrapping_add(r[8].wrapping_mul(0x1u64)))
                        & 0xffff_ffff;
                }
                // 14009e51e  498d42ff  LEA RAX,[R10 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[10]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009e522  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009e525  0f8565ffffff  JNZ 0x14009e490
                pc = if !f.z { 0x14009e490 } else { 0x14009e52b };
                continue;
            }
            0x14009e52b => {
                // 14009e52b  8b6c2440  MOV EBP,dword ptr [RSP + 0x40]
                {
                    r[5] =
                        (m.read::<32>(0u64.wrapping_add(r[4]).wrapping_add(0x40u64))) & 0xffff_ffff;
                }
                // 14009e52f  488b05422c3200  MOV RAX,qword ptr [0x1403c1178]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                }
                // 14009e536  4c897c2428  MOV qword ptr [RSP + 0x28],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[15]);
                }
                // 14009e53b  895c2430  MOV dword ptr [RSP + 0x30],EBX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x30u64),
                        r[3] & 0xffffffffu64,
                    );
                }
                pc = 0x14009e53f;
            }
            0x14009e53f => {
                // 14009e53f  4c8b1df22b3200  MOV R11,qword ptr [0x1403c1138]
                {
                    r[11] = m.read::<64>(0u64.wrapping_add(0x1403c1138u64));
                }
                // 14009e546  488b350b2c3200  MOV RSI,qword ptr [0x1403c1158]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1158u64));
                }
                // 14009e54d  4d85db  TEST R11,R11
                {
                    let v = r[11];
                    f.logic(v, 64);
                }
                // 14009e550  0f84d5000000  JZ 0x14009e62b
                pc = if f.z { 0x14009e62b } else { 0x14009e556 };
                continue;
            }
            0x14009e556 => {
                // 14009e556  4c8b642450  MOV R12,qword ptr [RSP + 0x50]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64));
                }
                // 14009e55b  4c8d3d9e1af6ff  LEA R15,[0x140000000]
                {
                    r[15] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009e562  488b9c2480000000  MOV RBX,qword ptr [RSP + 0x80]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x80u64));
                }
                // 14009e56a  488bfe  MOV RDI,RSI
                {
                    r[7] = r[6];
                }
                // 14009e56d  4833f8  XOR RDI,RAX
                {
                    let v = r[7] ^ r[0];
                    f.logic(v, 64);
                    r[7] = v;
                }
                pc = 0x14009e570;
            }
            0x14009e570 => {
                // 14009e570  490fbcc3  BSF RAX,R11
                {
                    let v = r[11];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009e574  4c63d0  MOVSXD R10,EAX
                {
                    r[10] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009e577  89059b0d2600  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009e57d  4b8b84d7f0040f00  MOV RAX,qword ptr [R15 + R10*0x8 + 0xf04f0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[10].wrapping_mul(0x8u64))
                            .wrapping_add(0xf04f0u64),
                    );
                }
                // 14009e585  438b8c9700090f00  MOV ECX,dword ptr [R15 + R10*0x4 + 0xf0900]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[10].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0900u64),
                    )) & 0xffff_ffff;
                }
                // 14009e58d  4f8b84d7f0e00e00  MOV R8,qword ptr [R15 + R10*0x8 + 0xee0f0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[10].wrapping_mul(0x8u64))
                            .wrapping_add(0xee0f0u64),
                    );
                }
                // 14009e595  4c23c7  AND R8,RDI
                {
                    let v = r[8] & r[7];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009e598  4f0faf84d7f0000f00  IMUL R8,qword ptr [R15 + R10*0x8 + 0xf00f0]
                {
                    r[8] = (r[8]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[15])
                                .wrapping_add(r[10].wrapping_mul(0x8u64))
                                .wrapping_add(0xf00f0u64),
                        ),
                    );
                }
                // 14009e5a1  49d3e8  SHR R8,CL
                {
                    let v = r[8];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 14009e5a4  4a8b0cc0  MOV RCX,qword ptr [RAX + R8*0x8]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[8].wrapping_mul(0x8u64)),
                    );
                }
                // 14009e5a8  8d4511  LEA EAX,[RBP + 0x11]
                {
                    r[0] = (0u64.wrapping_add(r[5]).wrapping_add(0x11u64)) & 0xffff_ffff;
                }
                // 14009e5ab  4f8b84d7e0521000  MOV R8,qword ptr [R15 + R10*0x8 + 0x1052e0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[10].wrapping_mul(0x8u64))
                            .wrapping_add(0x1052e0u64),
                    );
                }
                // 14009e5b3  4c0bf1  OR R14,RCX
                {
                    let v = r[14] | r[1];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 14009e5b6  4985cc  TEST R12,RCX
                {
                    let v = r[12] & r[1];
                    f.logic(v, 64);
                }
                // 14009e5b9  0f44c5  CMOVZ EAX,EBP
                {
                    if f.z {
                        r[0] = (r[5] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e5bc  4823cb  AND RCX,RBX
                {
                    let v = r[1] & r[3];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009e5bf  8be8  MOV EBP,EAX
                {
                    r[5] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e5c1  49c1e204  SHL R10,0x4
                {
                    let v = r[10];
                    let n = 4u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[10] = v;
                    }
                }
                // 14009e5c5  4c0fafc1  IMUL R8,RCX
                {
                    r[8] = (r[8]).wrapping_mul(r[1]);
                }
                // 14009e5c9  f3480fb8c1  POPCNT RAX,RCX
                {
                    let v = r[1];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009e5ce  4469c81b001b00  IMUL R9D,EAX,0x1b001b
                {
                    r[9] = ((r[0] & 0xffffffffu64).wrapping_mul(0x1b001bu64)) & 0xffff_ffff;
                }
                // 14009e5d5  498bc3  MOV RAX,R11
                {
                    r[0] = r[11];
                }
                // 14009e5d8  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 14009e5db  49c1e83c  SHR R8,0x3c
                {
                    let v = r[8];
                    let n = 60u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 14009e5df  49234520  AND RAX,qword ptr [R13 + 0x20]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x20u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e5e3  4d03c2  ADD R8,R10
                {
                    let v = f.arithmetic(r[8], r[10], 64, false);
                    r[8] = v;
                }
                // 14009e5e6  4985c3  TEST R11,RAX
                {
                    let v = r[11] & r[0];
                    f.logic(v, 64);
                }
                // 14009e5e9  8d82e2ffd2ff  LEA EAX,[RDX + -0x2d001e]
                {
                    r[0] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffd2ffe2u64)) & 0xffff_ffff;
                }
                // 14009e5ef  0f45c2  CMOVNZ EAX,EDX
                {
                    if !f.z {
                        r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e5f2  8bd0  MOV EDX,EAX
                {
                    r[2] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e5f4  498d43ff  LEA RAX,[R11 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[11]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009e5f8  432b9487e0541000  SUB EDX,dword ptr [R15 + R8*0x4 + 0x1054e0]
                {
                    let v = f.arithmetic(
                        r[2] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[15])
                                .wrapping_add(r[8].wrapping_mul(0x4u64))
                                .wrapping_add(0x1054e0u64),
                        ),
                        32,
                        true,
                    );
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009e600  412bd1  SUB EDX,R9D
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[9] & 0xffffffffu64, 32, true);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009e603  4c23d8  AND R11,RAX
                {
                    let v = r[11] & r[0];
                    f.logic(v, 64);
                    r[11] = v;
                }
                // 14009e606  0f8564ffffff  JNZ 0x14009e570
                pc = if !f.z { 0x14009e570 } else { 0x14009e60c };
                continue;
            }
            0x14009e60c => {
                // 14009e60c  8b5c2430  MOV EBX,dword ptr [RSP + 0x30]
                {
                    r[3] =
                        (m.read::<32>(0u64.wrapping_add(r[4]).wrapping_add(0x30u64))) & 0xffff_ffff;
                }
                // 14009e610  4c8b7c2428  MOV R15,qword ptr [RSP + 0x28]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64));
                }
                // 14009e615  488bbc2490000000  MOV RDI,qword ptr [RSP + 0x90]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x90u64));
                }
                // 14009e61d  4c8b642468  MOV R12,qword ptr [RSP + 0x68]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x68u64));
                }
                // 14009e622  4c89742438  MOV qword ptr [RSP + 0x38],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64), r[14]);
                }
                // 14009e627  896c2440  MOV dword ptr [RSP + 0x40],EBP
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x40u64),
                        r[5] & 0xffffffffu64,
                    );
                }
                pc = 0x14009e62b;
            }
            0x14009e62b => {
                // 14009e62b  488b0d0e2b3200  MOV RCX,qword ptr [0x1403c1140]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(0x1403c1140u64));
                }
                // 14009e632  488b050f2b3200  MOV RAX,qword ptr [0x1403c1148]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                }
                // 14009e639  4c8bd9  MOV R11,RCX
                {
                    r[11] = r[1];
                }
                // 14009e63c  4c8b150d2b3200  MOV R10,qword ptr [0x1403c1150]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1150u64));
                }
                // 14009e643  480bc6  OR RAX,RSI
                {
                    let v = r[0] | r[6];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e646  4923c7  AND RAX,R15
                {
                    let v = r[0] & r[15];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e649  4809442458  OR qword ptr [RSP + 0x58],RAX
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64)) | r[0];
                    f.logic(v, 64);
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64), v);
                }
                // 14009e64e  488bc1  MOV RAX,RCX
                {
                    r[0] = r[1];
                }
                // 14009e651  490bc2  OR RAX,R10
                {
                    let v = r[0] | r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e654  4923c6  AND RAX,R14
                {
                    let v = r[0] & r[14];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e657  4809442448  OR qword ptr [RSP + 0x48],RAX
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64)) | r[0];
                    f.logic(v, 64);
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64), v);
                }
                // 14009e65c  4885c9  TEST RCX,RCX
                {
                    let v = r[1];
                    f.logic(v, 64);
                }
                // 14009e65f  0f8420020000  JZ 0x14009e885
                pc = if f.z { 0x14009e885 } else { 0x14009e665 };
                continue;
            }
            0x14009e665 => {
                // 14009e665  4c33150c2b3200  XOR R10,qword ptr [0x1403c1178]
                {
                    let v = r[10] ^ m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009e66c  498bcc  MOV RCX,R12
                {
                    r[1] = r[12];
                }
                // 14009e66f  488b442470  MOV RAX,qword ptr [RSP + 0x70]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64));
                }
                // 14009e674  48f7d1  NOT RCX
                {
                    let v = !(r[1]);
                    r[1] = v;
                }
                // 14009e677  48230d922a3200  AND RCX,qword ptr [0x1403c1110]
                {
                    let v = r[1] & m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009e67e  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009e681  482305902a3200  AND RAX,qword ptr [0x1403c1118]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e688  4923c4  AND RAX,R12
                {
                    let v = r[0] & r[12];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e68b  4c89542420  MOV qword ptr [RSP + 0x20],R10
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[10]);
                }
                // 14009e690  480bc1  OR RAX,RCX
                {
                    let v = r[0] | r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e693  48894c2430  MOV qword ptr [RSP + 0x30],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x30u64), r[1]);
                }
                // 14009e698  4889442428  MOV qword ptr [RSP + 0x28],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[0]);
                }
                // 14009e69d  4c8d255c19f6ff  LEA R12,[0x140000000]
                {
                    r[12] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009e6a4  4c8b742428  MOV R14,qword ptr [RSP + 0x28]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64));
                }
                // 14009e6a9  488bc6  MOV RAX,RSI
                {
                    r[0] = r[6];
                }
                // 14009e6ac  480bc7  OR RAX,RDI
                {
                    let v = r[0] | r[7];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e6af  4889442440  MOV qword ptr [RSP + 0x40],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x40u64), r[0]);
                }
                // 14009e6b4  0f1f4000  NOP dword ptr [RAX]
                // 14009e6b8  0f1f840000000000  NOP dword ptr [RAX + RAX*0x1]
                pc = 0x14009e6c0;
            }
            0x14009e6c0 => {
                // 14009e6c0  4d0fbccb  BSF R9,R11
                {
                    let v = r[11];
                    f.z = v == 0;
                    if v != 0 {
                        r[9] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009e6c4  4963c1  MOVSXD RAX,R9D
                {
                    r[0] = ((r[9] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009e6c7  44890d4a0c2600  MOV dword ptr [0x1402ff318],R9D
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[9] & 0xffffffffu64);
                }
                // 14009e6ce  418b8c8400080f00  MOV ECX,dword ptr [R12 + RAX*0x4 + 0xf0800]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0800u64),
                    )) & 0xffff_ffff;
                }
                // 14009e6d6  488d3cc500000000  LEA RDI,[RAX*0x8]
                {
                    r[7] = 0u64.wrapping_add(r[0].wrapping_mul(0x8u64));
                }
                // 14009e6de  4e8b8427f0de0e00  MOV R8,qword ptr [RDI + R12*0x1 + 0xedef0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xedef0u64),
                    );
                }
                // 14009e6e6  4a8bb427f0fe0e00  MOV RSI,qword ptr [RDI + R12*0x1 + 0xefef0]
                {
                    r[6] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xefef0u64),
                    );
                }
                // 14009e6ee  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009e6f1  4e8ba427f0020f00  MOV R12,qword ptr [RDI + R12*0x1 + 0xf02f0]
                {
                    r[12] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xf02f0u64),
                    );
                }
                // 14009e6f9  4923c2  AND RAX,R10
                {
                    let v = r[0] & r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e6fc  480fafc6  IMUL RAX,RSI
                {
                    r[0] = (r[0]).wrapping_mul(r[6]);
                }
                // 14009e700  48d3e8  SHR RAX,CL
                {
                    let v = r[0];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009e703  4d8b14c4  MOV R10,qword ptr [R12 + RAX*0x8]
                {
                    r[10] = m.read::<64>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x8u64)),
                    );
                }
                // 14009e707  8d4313  LEA EAX,[RBX + 0x13]
                {
                    r[0] = (0u64.wrapping_add(r[3]).wrapping_add(0x13u64)) & 0xffff_ffff;
                }
                // 14009e70a  4d0bfa  OR R15,R10
                {
                    let v = r[15] | r[10];
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009e70d  4c85542460  TEST qword ptr [RSP + 0x60],R10
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64)) & r[10];
                    f.logic(v, 64);
                }
                // 14009e712  0f44c3  CMOVZ EAX,EBX
                {
                    if f.z {
                        r[0] = (r[3] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e715  4c85542450  TEST qword ptr [RSP + 0x50],R10
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64)) & r[10];
                    f.logic(v, 64);
                }
                // 14009e71a  8bd8  MOV EBX,EAX
                {
                    r[3] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e71c  8d45fd  LEA EAX,[RBP + -0x3]
                {
                    r[0] =
                        (0u64.wrapping_add(r[5]).wrapping_add(0xfffffffffffffffdu64)) & 0xffff_ffff;
                }
                // 14009e71f  0f44c5  CMOVZ EAX,EBP
                {
                    if f.z {
                        r[0] = (r[5] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e722  4c8515172a3200  TEST qword ptr [0x1403c1140],R10
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1140u64)) & r[10];
                    f.logic(v, 64);
                }
                // 14009e729  8be8  MOV EBP,EAX
                {
                    r[5] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e72b  7410  JZ 0x14009e73d
                pc = if f.z { 0x14009e73d } else { 0x14009e72d };
                continue;
            }
            0x14009e72d => {
                // 14009e72d  48833d232a320000  CMP qword ptr [0x1403c1158],0x0
                {
                    f.arithmetic(
                        m.read::<64>(0u64.wrapping_add(0x1403c1158u64)),
                        0x0u64,
                        64,
                        true,
                    );
                }
                // 14009e735  7406  JZ 0x14009e73d
                pc = if f.z { 0x14009e73d } else { 0x14009e737 };
                continue;
            }
            0x14009e737 => {
                // 14009e737  81c22a001800  ADD EDX,0x18002a
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, 0x18002au64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                pc = 0x14009e73d;
            }
            0x14009e73d => {
                // 14009e73d  498bc6  MOV RAX,R14
                {
                    r[0] = r[14];
                }
                // 14009e740  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e743  480fafc6  IMUL RAX,RSI
                {
                    r[0] = (r[0]).wrapping_mul(r[6]);
                }
                // 14009e747  48d3e8  SHR RAX,CL
                {
                    let v = r[0];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009e74a  f3490fb804c4  POPCNT RAX,qword ptr [R12 + RAX*0x8]
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x8u64)),
                    );
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009e750  4469c009000800  IMUL R8D,EAX,0x80009
                {
                    r[8] = ((r[0] & 0xffffffffu64).wrapping_mul(0x80009u64)) & 0xffff_ffff;
                }
                // 14009e757  4c8d25a218f6ff  LEA R12,[0x140000000]
                {
                    r[12] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009e75e  488b442440  MOV RAX,qword ptr [RSP + 0x40]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x40u64));
                }
                // 14009e763  4923c2  AND RAX,R10
                {
                    let v = r[0] & r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e766  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009e76b  69c817001100  IMUL ECX,EAX,0x110017
                {
                    r[1] = ((r[0] & 0xffffffffu64).wrapping_mul(0x110017u64)) & 0xffff_ffff;
                }
                // 14009e771  03d1  ADD EDX,ECX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009e773  4a8b8c2780440f00  MOV RCX,qword ptr [RDI + R12*0x1 + 0xf4480]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4480u64),
                    );
                }
                // 14009e77b  4103d0  ADD EDX,R8D
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009e77e  48854c2430  TEST qword ptr [RSP + 0x30],RCX
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x30u64)) & r[1];
                    f.logic(v, 64);
                }
                // 14009e783  0f85c5000000  JNZ 0x14009e84e
                pc = if !f.z { 0x14009e84e } else { 0x14009e789 };
                continue;
            }
            0x14009e789 => {
                // 14009e789  488b3d98293200  MOV RDI,qword ptr [0x1403c1128]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(0x1403c1128u64));
                }
                // 14009e790  4c8b15a1293200  MOV R10,qword ptr [0x1403c1138]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1138u64));
                }
                // 14009e797  488bc7  MOV RAX,RDI
                {
                    r[0] = r[7];
                }
                // 14009e79a  4c8b0577293200  MOV R8,qword ptr [0x1403c1118]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 14009e7a1  490bc2  OR RAX,R10
                {
                    let v = r[0] | r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e7a4  490bc0  OR RAX,R8
                {
                    let v = r[0] | r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e7a7  49234510  AND RAX,qword ptr [R13 + 0x10]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x10u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e7ab  4885c1  TEST RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                }
                // 14009e7ae  7562  JNZ 0x14009e812
                pc = if !f.z { 0x14009e812 } else { 0x14009e7b0 };
                continue;
            }
            0x14009e7b0 => {
                // 14009e7b0  4c85c1  TEST RCX,R8
                {
                    let v = r[1] & r[8];
                    f.logic(v, 64);
                }
                // 14009e7b3  741b  JZ 0x14009e7d0
                pc = if f.z { 0x14009e7d0 } else { 0x14009e7b5 };
                continue;
            }
            0x14009e7b5 => {
                // 14009e7b5  498b4528  MOV RAX,qword ptr [R13 + 0x28]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x28u64));
                }
                // 14009e7b9  41ba46000a00  MOV R10D,0xa0046
                {
                    r[10] = (0xa0046u64) & 0xffff_ffff;
                }
                // 14009e7bf  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e7c2  4885c1  TEST RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                }
                // 14009e7c5  b883002f00  MOV EAX,0x2f0083
                {
                    r[0] = (0x2f0083u64) & 0xffff_ffff;
                }
                // 14009e7ca  410f45c2  CMOVNZ EAX,R10D
                {
                    if !f.z {
                        r[0] = (r[10] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e7ce  eb40  JMP 0x14009e810
                pc = 0x14009e810;
                continue;
            }
            0x14009e7d0 => {
                // 14009e7d0  4c8bc7  MOV R8,RDI
                {
                    r[8] = r[7];
                }
                // 14009e7d3  4d0bc2  OR R8,R10
                {
                    let v = r[8] | r[10];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009e7d6  4c8b542468  MOV R10,qword ptr [RSP + 0x68]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x68u64));
                }
                // 14009e7db  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009e7de  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e7e1  4985c2  TEST R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                }
                // 14009e7e4  741e  JZ 0x14009e804
                pc = if f.z { 0x14009e804 } else { 0x14009e7e6 };
                continue;
            }
            0x14009e7e6 => {
                // 14009e7e6  498b4520  MOV RAX,qword ptr [R13 + 0x20]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x20u64));
                }
                // 14009e7ea  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e7ed  41b87700a300  MOV R8D,0xa30077
                {
                    r[8] = (0xa30077u64) & 0xffff_ffff;
                }
                // 14009e7f3  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e7f6  4985c2  TEST R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                }
                // 14009e7f9  b852007100  MOV EAX,0x710052
                {
                    r[0] = (0x710052u64) & 0xffff_ffff;
                }
                // 14009e7fe  410f45c0  CMOVNZ EAX,R8D
                {
                    if !f.z {
                        r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e802  eb05  JMP 0x14009e809
                pc = 0x14009e809;
                continue;
            }
            0x14009e804 => {
                // 14009e804  b88800b700  MOV EAX,0xb70088
                {
                    r[0] = (0xb70088u64) & 0xffff_ffff;
                }
                pc = 0x14009e809;
            }
            0x14009e809 => {
                // 14009e809  4c8b0508293200  MOV R8,qword ptr [0x1403c1118]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x14009e810;
            }
            0x14009e810 => {
                // 14009e810  03d0  ADD EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                pc = 0x14009e812;
            }
            0x14009e812 => {
                // 14009e812  4c8b154f293200  MOV R10,qword ptr [0x1403c1168]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1168u64));
                }
                // 14009e819  8d8200009600  LEA EAX,[RDX + 0x960000]
                {
                    r[0] = (0u64.wrapping_add(r[2]).wrapping_add(0x960000u64)) & 0xffff_ffff;
                }
                // 14009e81f  4c85d1  TEST RCX,R10
                {
                    let v = r[1] & r[10];
                    f.logic(v, 64);
                }
                // 14009e822  0f44c2  CMOVZ EAX,EDX
                {
                    if f.z {
                        r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e825  4183e138  AND R9D,0x38
                {
                    let v = (r[9] & 0xffffffffu64 & 0x38u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 14009e829  8bd0  MOV EDX,EAX
                {
                    r[2] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e82b  8bc8  MOV ECX,EAX
                {
                    r[1] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e82d  4180f930  CMP R9B,0x30
                {
                    f.arithmetic(r[9] & 0xffu64, 0x30u64, 8, true);
                }
                // 14009e831  751b  JNZ 0x14009e84e
                pc = if !f.z { 0x14009e84e } else { 0x14009e833 };
                continue;
            }
            0x14009e833 => {
                // 14009e833  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009e836  49b8000000000000ffff  MOV R8,-0x1000000000000
                {
                    r[8] = 0xffff000000000000u64;
                }
                // 14009e840  490bc2  OR RAX,R10
                {
                    let v = r[0] | r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e843  4985c0  TEST R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                }
                // 14009e846  7406  JZ 0x14009e84e
                pc = if f.z { 0x14009e84e } else { 0x14009e848 };
                continue;
            }
            0x14009e848 => {
                // 14009e848  8d91f7004d00  LEA EDX,[RCX + 0x4d00f7]
                {
                    r[2] = (0u64.wrapping_add(r[1]).wrapping_add(0x4d00f7u64)) & 0xffff_ffff;
                }
                pc = 0x14009e84e;
            }
            0x14009e84e => {
                // 14009e84e  4c8b542420  MOV R10,qword ptr [RSP + 0x20]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                }
                // 14009e853  498d43ff  LEA RAX,[R11 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[11]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009e857  4c23d8  AND R11,RAX
                {
                    let v = r[11] & r[0];
                    f.logic(v, 64);
                    r[11] = v;
                }
                // 14009e85a  0f8560feffff  JNZ 0x14009e6c0
                pc = if !f.z { 0x14009e6c0 } else { 0x14009e860 };
                continue;
            }
            0x14009e860 => {
                // 14009e860  4c8b742438  MOV R14,qword ptr [RSP + 0x38]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64));
                }
                // 14009e865  488b35ec283200  MOV RSI,qword ptr [0x1403c1158]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1158u64));
                }
                // 14009e86c  4c8b642468  MOV R12,qword ptr [RSP + 0x68]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x68u64));
                }
                // 14009e871  4c8b15d8283200  MOV R10,qword ptr [0x1403c1150]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1150u64));
                }
                // 14009e878  4c897c2428  MOV qword ptr [RSP + 0x28],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[15]);
                }
                // 14009e87d  895c2430  MOV dword ptr [RSP + 0x30],EBX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x30u64),
                        r[3] & 0xffffffffu64,
                    );
                }
                // 14009e881  896c2440  MOV dword ptr [RSP + 0x40],EBP
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x40u64),
                        r[5] & 0xffffffffu64,
                    );
                }
                pc = 0x14009e885;
            }
            0x14009e885 => {
                // 14009e885  4c8b1dbc283200  MOV R11,qword ptr [0x1403c1148]
                {
                    r[11] = m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                }
                // 14009e88c  4d85db  TEST R11,R11
                {
                    let v = r[11];
                    f.logic(v, 64);
                }
                // 14009e88f  0f840b020000  JZ 0x14009eaa0
                pc = if f.z { 0x14009eaa0 } else { 0x14009e895 };
                continue;
            }
            0x14009e895 => {
                // 14009e895  488b4c2470  MOV RCX,qword ptr [RSP + 0x70]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64));
                }
                // 14009e89a  498bc4  MOV RAX,R12
                {
                    r[0] = r[12];
                }
                // 14009e89d  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009e8a0  4c8d255917f6ff  LEA R12,[0x140000000]
                {
                    r[12] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009e8a7  48230562283200  AND RAX,qword ptr [0x1403c1110]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e8ae  48f7d1  NOT RCX
                {
                    let v = !(r[1]);
                    r[1] = v;
                }
                // 14009e8b1  4823442470  AND RAX,qword ptr [RSP + 0x70]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e8b6  4c8bce  MOV R9,RSI
                {
                    r[9] = r[6];
                }
                // 14009e8b9  48230d58283200  AND RCX,qword ptr [0x1403c1118]
                {
                    let v = r[1] & m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009e8c0  4c330db1283200  XOR R9,qword ptr [0x1403c1178]
                {
                    let v = r[9] ^ m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009e8c7  480bc1  OR RAX,RCX
                {
                    let v = r[0] | r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e8ca  4889442420  MOV qword ptr [RSP + 0x20],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[0]);
                }
                // 14009e8cf  498bc2  MOV RAX,R10
                {
                    r[0] = r[10];
                }
                // 14009e8d2  480b842480000000  OR RAX,qword ptr [RSP + 0x80]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x80u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e8da  4c8b7c2420  MOV R15,qword ptr [RSP + 0x20]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                }
                // 14009e8df  4889442438  MOV qword ptr [RSP + 0x38],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64), r[0]);
                }
                // 14009e8e4  4c894c2430  MOV qword ptr [RSP + 0x30],R9
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x30u64), r[9]);
                }
                // 14009e8e9  48894c2440  MOV qword ptr [RSP + 0x40],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x40u64), r[1]);
                }
                // 14009e8ee  6690  NOP
                pc = 0x14009e8f0;
            }
            0x14009e8f0 => {
                // 14009e8f0  4d0fbcd3  BSF R10,R11
                {
                    let v = r[11];
                    f.z = v == 0;
                    if v != 0 {
                        r[10] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009e8f4  4963c2  MOVSXD RAX,R10D
                {
                    r[0] = ((r[10] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009e8f7  4489151a0a2600  MOV dword ptr [0x1402ff318],R10D
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[10] & 0xffffffffu64);
                }
                // 14009e8fe  418b8c8400080f00  MOV ECX,dword ptr [R12 + RAX*0x4 + 0xf0800]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0800u64),
                    )) & 0xffff_ffff;
                }
                // 14009e906  488d3cc500000000  LEA RDI,[RAX*0x8]
                {
                    r[7] = 0u64.wrapping_add(r[0].wrapping_mul(0x8u64));
                }
                // 14009e90e  4e8b8427f0de0e00  MOV R8,qword ptr [RDI + R12*0x1 + 0xedef0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xedef0u64),
                    );
                }
                // 14009e916  4a8bb427f0fe0e00  MOV RSI,qword ptr [RDI + R12*0x1 + 0xefef0]
                {
                    r[6] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xefef0u64),
                    );
                }
                // 14009e91e  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009e921  4e8ba427f0020f00  MOV R12,qword ptr [RDI + R12*0x1 + 0xf02f0]
                {
                    r[12] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xf02f0u64),
                    );
                }
                // 14009e929  4923c1  AND RAX,R9
                {
                    let v = r[0] & r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e92c  480fafc6  IMUL RAX,RSI
                {
                    r[0] = (r[0]).wrapping_mul(r[6]);
                }
                // 14009e930  48d3e8  SHR RAX,CL
                {
                    let v = r[0];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009e933  4d8b0cc4  MOV R9,qword ptr [R12 + RAX*0x8]
                {
                    r[9] = m.read::<64>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x8u64)),
                    );
                }
                // 14009e937  8d4513  LEA EAX,[RBP + 0x13]
                {
                    r[0] = (0u64.wrapping_add(r[5]).wrapping_add(0x13u64)) & 0xffff_ffff;
                }
                // 14009e93a  4d0bf1  OR R14,R9
                {
                    let v = r[14] | r[9];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 14009e93d  4c854c2450  TEST qword ptr [RSP + 0x50],R9
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64)) & r[9];
                    f.logic(v, 64);
                }
                // 14009e942  0f44c5  CMOVZ EAX,EBP
                {
                    if f.z {
                        r[0] = (r[5] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e945  4c854c2460  TEST qword ptr [RSP + 0x60],R9
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64)) & r[9];
                    f.logic(v, 64);
                }
                // 14009e94a  8be8  MOV EBP,EAX
                {
                    r[5] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e94c  8d43fd  LEA EAX,[RBX + -0x3]
                {
                    r[0] =
                        (0u64.wrapping_add(r[3]).wrapping_add(0xfffffffffffffffdu64)) & 0xffff_ffff;
                }
                // 14009e94f  0f44c3  CMOVZ EAX,EBX
                {
                    if f.z {
                        r[0] = (r[3] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e952  4c850def273200  TEST qword ptr [0x1403c1148],R9
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1148u64)) & r[9];
                    f.logic(v, 64);
                }
                // 14009e959  8bd8  MOV EBX,EAX
                {
                    r[3] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009e95b  7410  JZ 0x14009e96d
                pc = if f.z { 0x14009e96d } else { 0x14009e95d };
                continue;
            }
            0x14009e95d => {
                // 14009e95d  48833deb27320000  CMP qword ptr [0x1403c1150],0x0
                {
                    f.arithmetic(
                        m.read::<64>(0u64.wrapping_add(0x1403c1150u64)),
                        0x0u64,
                        64,
                        true,
                    );
                }
                // 14009e965  7406  JZ 0x14009e96d
                pc = if f.z { 0x14009e96d } else { 0x14009e967 };
                continue;
            }
            0x14009e967 => {
                // 14009e967  81ea2a001800  SUB EDX,0x18002a
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, 0x18002au64, 32, true);
                    r[2] = (v) & 0xffff_ffff;
                }
                pc = 0x14009e96d;
            }
            0x14009e96d => {
                // 14009e96d  4c234c2438  AND R9,qword ptr [RSP + 0x38]
                {
                    let v = r[9] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64));
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009e972  498bc7  MOV RAX,R15
                {
                    r[0] = r[15];
                }
                // 14009e975  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e978  480fafc6  IMUL RAX,RSI
                {
                    r[0] = (r[0]).wrapping_mul(r[6]);
                }
                // 14009e97c  48d3e8  SHR RAX,CL
                {
                    let v = r[0];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009e97f  f3490fb804c4  POPCNT RAX,qword ptr [R12 + RAX*0x8]
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x8u64)),
                    );
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009e985  4469c0f7fff7ff  IMUL R8D,EAX,-0x80009
                {
                    r[8] =
                        ((r[0] & 0xffffffffu64).wrapping_mul(0xfffffffffff7fff7u64)) & 0xffff_ffff;
                }
                // 14009e98c  4c8d256d16f6ff  LEA R12,[0x140000000]
                {
                    r[12] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009e993  f3490fb8c1  POPCNT RAX,R9
                {
                    let v = r[9];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009e998  69c817001100  IMUL ECX,EAX,0x110017
                {
                    r[1] = ((r[0] & 0xffffffffu64).wrapping_mul(0x110017u64)) & 0xffff_ffff;
                }
                // 14009e99e  442bc1  SUB R8D,ECX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, true);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009e9a1  4a8b8c2780420f00  MOV RCX,qword ptr [RDI + R12*0x1 + 0xf4280]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[12].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4280u64),
                    );
                }
                // 14009e9a9  4103d0  ADD EDX,R8D
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009e9ac  48854c2440  TEST qword ptr [RSP + 0x40],RCX
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x40u64)) & r[1];
                    f.logic(v, 64);
                }
                // 14009e9b1  0f85c5000000  JNZ 0x14009ea7c
                pc = if !f.z { 0x14009ea7c } else { 0x14009e9b7 };
                continue;
            }
            0x14009e9b7 => {
                // 14009e9b7  4c8b2562273200  MOV R12,qword ptr [0x1403c1120]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(0x1403c1120u64));
                }
                // 14009e9be  4c8b0d6b273200  MOV R9,qword ptr [0x1403c1130]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(0x1403c1130u64));
                }
                // 14009e9c5  498bc4  MOV RAX,R12
                {
                    r[0] = r[12];
                }
                // 14009e9c8  4c8b0541273200  MOV R8,qword ptr [0x1403c1110]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 14009e9cf  490bc1  OR RAX,R9
                {
                    let v = r[0] | r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e9d2  490bc0  OR RAX,R8
                {
                    let v = r[0] | r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e9d5  49234508  AND RAX,qword ptr [R13 + 0x8]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x8u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e9d9  4885c1  TEST RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                }
                // 14009e9dc  7562  JNZ 0x14009ea40
                pc = if !f.z { 0x14009ea40 } else { 0x14009e9de };
                continue;
            }
            0x14009e9de => {
                // 14009e9de  4c85c1  TEST RCX,R8
                {
                    let v = r[1] & r[8];
                    f.logic(v, 64);
                }
                // 14009e9e1  741b  JZ 0x14009e9fe
                pc = if f.z { 0x14009e9fe } else { 0x14009e9e3 };
                continue;
            }
            0x14009e9e3 => {
                // 14009e9e3  498b4520  MOV RAX,qword ptr [R13 + 0x20]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x20u64));
                }
                // 14009e9e7  41b946000a00  MOV R9D,0xa0046
                {
                    r[9] = (0xa0046u64) & 0xffff_ffff;
                }
                // 14009e9ed  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009e9f0  4885c1  TEST RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                }
                // 14009e9f3  b883002f00  MOV EAX,0x2f0083
                {
                    r[0] = (0x2f0083u64) & 0xffff_ffff;
                }
                // 14009e9f8  410f45c1  CMOVNZ EAX,R9D
                {
                    if !f.z {
                        r[0] = (r[9] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009e9fc  eb40  JMP 0x14009ea3e
                pc = 0x14009ea3e;
                continue;
            }
            0x14009e9fe => {
                // 14009e9fe  4d8bc4  MOV R8,R12
                {
                    r[8] = r[12];
                }
                // 14009ea01  4d0bc1  OR R8,R9
                {
                    let v = r[8] | r[9];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009ea04  4c8b4c2470  MOV R9,qword ptr [RSP + 0x70]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64));
                }
                // 14009ea09  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009ea0c  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ea0f  4985c1  TEST R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                }
                // 14009ea12  741e  JZ 0x14009ea32
                pc = if f.z { 0x14009ea32 } else { 0x14009ea14 };
                continue;
            }
            0x14009ea14 => {
                // 14009ea14  498b4528  MOV RAX,qword ptr [R13 + 0x28]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x28u64));
                }
                // 14009ea18  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ea1b  41b87700a300  MOV R8D,0xa30077
                {
                    r[8] = (0xa30077u64) & 0xffff_ffff;
                }
                // 14009ea21  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ea24  4985c1  TEST R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                }
                // 14009ea27  b852007100  MOV EAX,0x710052
                {
                    r[0] = (0x710052u64) & 0xffff_ffff;
                }
                // 14009ea2c  410f45c0  CMOVNZ EAX,R8D
                {
                    if !f.z {
                        r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009ea30  eb05  JMP 0x14009ea37
                pc = 0x14009ea37;
                continue;
            }
            0x14009ea32 => {
                // 14009ea32  b88800b700  MOV EAX,0xb70088
                {
                    r[0] = (0xb70088u64) & 0xffff_ffff;
                }
                pc = 0x14009ea37;
            }
            0x14009ea37 => {
                // 14009ea37  4c8b05d2263200  MOV R8,qword ptr [0x1403c1110]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                pc = 0x14009ea3e;
            }
            0x14009ea3e => {
                // 14009ea3e  2bd0  SUB EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[2] = (v) & 0xffff_ffff;
                }
                pc = 0x14009ea40;
            }
            0x14009ea40 => {
                // 14009ea40  4c8b0d19273200  MOV R9,qword ptr [0x1403c1160]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(0x1403c1160u64));
                }
                // 14009ea47  8d8200006aff  LEA EAX,[RDX + -0x960000]
                {
                    r[0] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xffffffffff6a0000u64)) & 0xffff_ffff;
                }
                // 14009ea4d  4985c9  TEST R9,RCX
                {
                    let v = r[9] & r[1];
                    f.logic(v, 64);
                }
                // 14009ea50  4c8d25a915f6ff  LEA R12,[0x140000000]
                {
                    r[12] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009ea57  0f44c2  CMOVZ EAX,EDX
                {
                    if f.z {
                        r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009ea5a  4180e238  AND R10B,0x38
                {
                    let v = (r[10] & 0xffu64 & 0x38u64) & 0xffu64;
                    f.logic(v, 8);
                    r[10] = (r[10] & !0xffu64) | ((v) & 0xffu64);
                }
                // 14009ea5e  8bd0  MOV EDX,EAX
                {
                    r[2] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009ea60  8bc8  MOV ECX,EAX
                {
                    r[1] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009ea62  4180fa08  CMP R10B,0x8
                {
                    f.arithmetic(r[10] & 0xffu64, 0x8u64, 8, true);
                }
                // 14009ea66  7514  JNZ 0x14009ea7c
                pc = if !f.z { 0x14009ea7c } else { 0x14009ea68 };
                continue;
            }
            0x14009ea68 => {
                // 14009ea68  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009ea6b  490bc1  OR RAX,R9
                {
                    let v = r[0] | r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ea6e  0fb7c0  MOVZX EAX,AX
                {
                    r[0] = (r[0] & 0xffffu64) & 0xffff_ffff;
                }
                // 14009ea71  4885c0  TEST RAX,RAX
                {
                    let v = r[0];
                    f.logic(v, 64);
                }
                // 14009ea74  7406  JZ 0x14009ea7c
                pc = if f.z { 0x14009ea7c } else { 0x14009ea76 };
                continue;
            }
            0x14009ea76 => {
                // 14009ea76  8d9109ffb2ff  LEA EDX,[RCX + -0x4d00f7]
                {
                    r[2] =
                        (0u64.wrapping_add(r[1]).wrapping_add(0xffffffffffb2ff09u64)) & 0xffff_ffff;
                }
                pc = 0x14009ea7c;
            }
            0x14009ea7c => {
                // 14009ea7c  4c8b4c2430  MOV R9,qword ptr [RSP + 0x30]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x30u64));
                }
                // 14009ea81  498d43ff  LEA RAX,[R11 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[11]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009ea85  4c23d8  AND R11,RAX
                {
                    let v = r[11] & r[0];
                    f.logic(v, 64);
                    r[11] = v;
                }
                // 14009ea88  0f8562feffff  JNZ 0x14009e8f0
                pc = if !f.z { 0x14009e8f0 } else { 0x14009ea8e };
                continue;
            }
            0x14009ea8e => {
                // 14009ea8e  4c8b7c2428  MOV R15,qword ptr [RSP + 0x28]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64));
                }
                // 14009ea93  4c89742438  MOV qword ptr [RSP + 0x38],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64), r[14]);
                }
                // 14009ea98  896c2440  MOV dword ptr [RSP + 0x40],EBP
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x40u64),
                        r[5] & 0xffffffffu64,
                    );
                }
                // 14009ea9c  895c2430  MOV dword ptr [RSP + 0x30],EBX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x30u64),
                        r[3] & 0xffffffffu64,
                    );
                }
                pc = 0x14009eaa0;
            }
            0x14009eaa0 => {
                // 14009eaa0  488b4c2458  MOV RCX,qword ptr [RSP + 0x58]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64));
                }
                // 14009eaa5  33ff  XOR EDI,EDI
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 14009eaa7  488b35a2263200  MOV RSI,qword ptr [0x1403c1150]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1150u64));
                }
                // 14009eaae  4c8b442448  MOV R8,qword ptr [RSP + 0x48]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64));
                }
                // 14009eab3  488b059e263200  MOV RAX,qword ptr [0x1403c1158]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1158u64));
                }
                // 14009eaba  4923c7  AND RAX,R15
                {
                    let v = r[0] & r[15];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009eabd  480bc8  OR RCX,RAX
                {
                    let v = r[1] | r[0];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009eac0  488bc6  MOV RAX,RSI
                {
                    r[0] = r[6];
                }
                // 14009eac3  f34c0fb8d1  POPCNT R10,RCX
                {
                    let v = r[1];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[10] = u64::from(v.count_ones());
                }
                // 14009eac8  4923c6  AND RAX,R14
                {
                    let v = r[0] & r[14];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009eacb  48894c2458  MOV qword ptr [RSP + 0x58],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64), r[1]);
                }
                // 14009ead0  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009ead3  8bcf  MOV ECX,EDI
                {
                    r[1] = (r[7] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009ead5  f34d0fb8c8  POPCNT R9,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[9] = u64::from(v.count_ones());
                }
                // 14009eada  4c89442448  MOV qword ptr [RSP + 0x48],R8
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64), r[8]);
                }
                // 14009eadf  458d42ff  LEA R8D,[R10 + -0x1]
                {
                    r[8] = (0u64.wrapping_add(r[10]).wrapping_add(0xffffffffffffffffu64))
                        & 0xffff_ffff;
                }
                // 14009eae3  418d41ff  LEA EAX,[R9 + -0x1]
                {
                    r[0] =
                        (0u64.wrapping_add(r[9]).wrapping_add(0xffffffffffffffffu64)) & 0xffff_ffff;
                }
                // 14009eae7  85c0  TEST EAX,EAX
                {
                    let v = (r[0] & 0xffffffffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                }
                // 14009eae9  0f4fc8  CMOVG ECX,EAX
                {
                    if !f.z && f.s == f.o {
                        r[1] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009eaec  4585c0  TEST R8D,R8D
                {
                    let v = (r[8] & 0xffffffffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                }
                // 14009eaef  8bc7  MOV EAX,EDI
                {
                    r[0] = (r[7] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009eaf1  410f4fc0  CMOVG EAX,R8D
                {
                    if !f.z && f.s == f.o {
                        r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009eaf5  452bd1  SUB R10D,R9D
                {
                    let v = f.arithmetic(r[10] & 0xffffffffu64, r[9] & 0xffffffffu64, 32, true);
                    r[10] = (v) & 0xffff_ffff;
                }
                // 14009eaf8  2bc1  SUB EAX,ECX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009eafa  4569da0e01b400  IMUL R11D,R10D,0xb4010e
                {
                    r[11] = ((r[10] & 0xffffffffu64).wrapping_mul(0xb4010eu64)) & 0xffff_ffff;
                }
                // 14009eb01  69c89402b801  IMUL ECX,EAX,0x1b80294
                {
                    r[1] = ((r[0] & 0xffffffffu64).wrapping_mul(0x1b80294u64)) & 0xffff_ffff;
                }
                // 14009eb07  4403da  ADD R11D,EDX
                {
                    let v = f.arithmetic(r[11] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009eb0a  4403d9  ADD R11D,ECX
                {
                    let v = f.arithmetic(r[11] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009eb0d  498bcf  MOV RCX,R15
                {
                    r[1] = r[15];
                }
                // 14009eb10  48894c2420  MOV qword ptr [RSP + 0x20],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[1]);
                }
                // 14009eb15  4885f6  TEST RSI,RSI
                {
                    let v = r[6];
                    f.logic(v, 64);
                }
                // 14009eb18  0f8443010000  JZ 0x14009ec61
                pc = if f.z { 0x14009ec61 } else { 0x14009eb1e };
                continue;
            }
            0x14009eb1e => {
                // 14009eb1e  4c8b25e3253200  MOV R12,qword ptr [0x1403c1108]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                }
                // 14009eb25  498bc6  MOV RAX,R14
                {
                    r[0] = r[14];
                }
                // 14009eb28  488b3de9253200  MOV RDI,qword ptr [0x1403c1118]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 14009eb2f  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009eb32  4c8b2d07263200  MOV R13,qword ptr [0x1403c1140]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(0x1403c1140u64));
                }
                // 14009eb39  4c33e7  XOR R12,RDI
                {
                    let v = r[12] ^ r[7];
                    f.logic(v, 64);
                    r[12] = v;
                }
                // 14009eb3c  4c332d35263200  XOR R13,qword ptr [0x1403c1178]
                {
                    let v = r[13] ^ m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                    f.logic(v, 64);
                    r[13] = v;
                }
                // 14009eb43  4c0be0  OR R12,RAX
                {
                    let v = r[12] | r[0];
                    f.logic(v, 64);
                    r[12] = v;
                }
                // 14009eb46  488b05b3253200  MOV RAX,qword ptr [0x1403c1100]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1100u64));
                }
                // 14009eb4d  480b442468  OR RAX,qword ptr [RSP + 0x68]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x68u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009eb52  4c8b350f263200  MOV R14,qword ptr [0x1403c1168]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(0x1403c1168u64));
                }
                // 14009eb59  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009eb5c  488b6c2460  MOV RBP,qword ptr [RSP + 0x60]
                {
                    r[5] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64));
                }
                // 14009eb61  4c23e0  AND R12,RAX
                {
                    let v = r[12] & r[0];
                    f.logic(v, 64);
                    r[12] = v;
                }
                // 14009eb64  0f1f4000  NOP dword ptr [RAX]
                // 14009eb68  0f1f840000000000  NOP dword ptr [RAX + RAX*0x1]
                pc = 0x14009eb70;
            }
            0x14009eb70 => {
                // 14009eb70  4c0fbcd6  BSF R10,RSI
                {
                    let v = r[6];
                    f.z = v == 0;
                    if v != 0 {
                        r[10] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009eb74  488d158514f6ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009eb7b  4963c2  MOVSXD RAX,R10D
                {
                    r[0] = ((r[10] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009eb7e  44891593072600  MOV dword ptr [0x1402ff318],R10D
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[10] & 0xffffffffu64);
                }
                // 14009eb85  8b8c8200080f00  MOV ECX,dword ptr [RDX + RAX*0x4 + 0xf0800]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0800u64),
                    )) & 0xffff_ffff;
                }
                // 14009eb8c  4c8b8cc2f0de0e00  MOV R9,qword ptr [RDX + RAX*0x8 + 0xedef0]
                {
                    r[9] = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xedef0u64),
                    );
                }
                // 14009eb94  4c8b84c2f0020f00  MOV R8,qword ptr [RDX + RAX*0x8 + 0xf02f0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf02f0u64),
                    );
                }
                // 14009eb9c  4d23cd  AND R9,R13
                {
                    let v = r[9] & r[13];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009eb9f  4c0faf8cc2f0fe0e00  IMUL R9,qword ptr [RDX + RAX*0x8 + 0xefef0]
                {
                    r[9] = (r[9]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[2])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xefef0u64),
                        ),
                    );
                }
                // 14009eba8  488b94c2f0e00e00  MOV RDX,qword ptr [RDX + RAX*0x8 + 0xee0f0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xee0f0u64),
                    );
                }
                // 14009ebb0  482315c1253200  AND RDX,qword ptr [0x1403c1178]
                {
                    let v = r[2] & m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 14009ebb7  49d3e9  SHR R9,CL
                {
                    let v = r[9];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 14009ebba  488d0d3f14f6ff  LEA RCX,[0x140000000]
                {
                    r[1] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009ebc1  480faf94c1f0000f00  IMUL RDX,qword ptr [RCX + RAX*0x8 + 0xf00f0]
                {
                    r[2] = (r[2]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xf00f0u64),
                        ),
                    );
                }
                // 14009ebca  8b8c8100090f00  MOV ECX,dword ptr [RCX + RAX*0x4 + 0xf0900]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0900u64),
                    )) & 0xffff_ffff;
                }
                // 14009ebd1  48d3ea  SHR RDX,CL
                {
                    let v = r[2];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 14009ebd4  488d0d2514f6ff  LEA RCX,[0x140000000]
                {
                    r[1] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009ebdb  488b84c1f0040f00  MOV RAX,qword ptr [RCX + RAX*0x8 + 0xf04f0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf04f0u64),
                    );
                }
                // 14009ebe3  4b8b0cc8  MOV RCX,qword ptr [R8 + R9*0x8]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[9].wrapping_mul(0x8u64)),
                    );
                }
                // 14009ebe7  480b0cd0  OR RCX,qword ptr [RAX + RDX*0x8]
                {
                    let v = r[1]
                        | m.read::<64>(
                            0u64.wrapping_add(r[0])
                                .wrapping_add(r[2].wrapping_mul(0x8u64)),
                        );
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009ebeb  8d4311  LEA EAX,[RBX + 0x11]
                {
                    r[0] = (0u64.wrapping_add(r[3]).wrapping_add(0x11u64)) & 0xffff_ffff;
                }
                // 14009ebee  4c0bf9  OR R15,RCX
                {
                    let v = r[15] | r[1];
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009ebf1  4885cd  TEST RBP,RCX
                {
                    let v = r[5] & r[1];
                    f.logic(v, 64);
                }
                // 14009ebf4  0f44c3  CMOVZ EAX,EBX
                {
                    if f.z {
                        r[0] = (r[3] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009ebf7  4183e238  AND R10D,0x38
                {
                    let v = (r[10] & 0xffffffffu64 & 0x38u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[10] = (v) & 0xffff_ffff;
                }
                // 14009ebfb  8bd8  MOV EBX,EAX
                {
                    r[3] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009ebfd  498bc4  MOV RAX,R12
                {
                    r[0] = r[12];
                }
                // 14009ec00  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ec03  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009ec08  69c81d001b00  IMUL ECX,EAX,0x1b001d
                {
                    r[1] = ((r[0] & 0xffffffffu64).wrapping_mul(0x1b001du64)) & 0xffff_ffff;
                }
                // 14009ec0e  4403d9  ADD R11D,ECX
                {
                    let v = f.arithmetic(r[11] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009ec11  4180fa30  CMP R10B,0x30
                {
                    f.arithmetic(r[10] & 0xffu64, 0x30u64, 8, true);
                }
                // 14009ec15  751c  JNZ 0x14009ec33
                pc = if !f.z { 0x14009ec33 } else { 0x14009ec17 };
                continue;
            }
            0x14009ec17 => {
                // 14009ec17  488bc7  MOV RAX,RDI
                {
                    r[0] = r[7];
                }
                // 14009ec1a  48b9000000000000ffff  MOV RCX,-0x1000000000000
                {
                    r[1] = 0xffff000000000000u64;
                }
                // 14009ec24  490bc6  OR RAX,R14
                {
                    let v = r[0] | r[14];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ec27  4885c1  TEST RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                }
                // 14009ec2a  7407  JZ 0x14009ec33
                pc = if f.z { 0x14009ec33 } else { 0x14009ec2c };
                continue;
            }
            0x14009ec2c => {
                // 14009ec2c  4181c32c010000  ADD R11D,0x12c
                {
                    let v = f.arithmetic(r[11] & 0xffffffffu64, 0x12cu64, 32, false);
                    r[11] = (v) & 0xffff_ffff;
                }
                pc = 0x14009ec33;
            }
            0x14009ec33 => {
                // 14009ec33  488d46ff  LEA RAX,[RSI + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[6]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009ec37  4823f0  AND RSI,RAX
                {
                    let v = r[6] & r[0];
                    f.logic(v, 64);
                    r[6] = v;
                }
                // 14009ec3a  0f8530ffffff  JNZ 0x14009eb70
                pc = if !f.z { 0x14009eb70 } else { 0x14009ec40 };
                continue;
            }
            0x14009ec40 => {
                // 14009ec40  8b6c2440  MOV EBP,dword ptr [RSP + 0x40]
                {
                    r[5] =
                        (m.read::<32>(0u64.wrapping_add(r[4]).wrapping_add(0x40u64))) & 0xffff_ffff;
                }
                // 14009ec44  33ff  XOR EDI,EDI
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 14009ec46  4c8b742438  MOV R14,qword ptr [RSP + 0x38]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64));
                }
                // 14009ec4b  4c8bac2488000000  MOV R13,qword ptr [RSP + 0x88]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x88u64));
                }
                // 14009ec53  488b4c2420  MOV RCX,qword ptr [RSP + 0x20]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                }
                // 14009ec58  4c897c2428  MOV qword ptr [RSP + 0x28],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[15]);
                }
                // 14009ec5d  895c2430  MOV dword ptr [RSP + 0x30],EBX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[4]).wrapping_add(0x30u64),
                        r[3] & 0xffffffffu64,
                    );
                }
                pc = 0x14009ec61;
            }
            0x14009ec61 => {
                // 14009ec61  488b35f0243200  MOV RSI,qword ptr [0x1403c1158]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1158u64));
                }
                // 14009ec68  4885f6  TEST RSI,RSI
                {
                    let v = r[6];
                    f.logic(v, 64);
                }
                // 14009ec6b  0f842f010000  JZ 0x14009eda0
                pc = if f.z { 0x14009eda0 } else { 0x14009ec71 };
                continue;
            }
            0x14009ec71 => {
                // 14009ec71  488b05d0243200  MOV RAX,qword ptr [0x1403c1148]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                }
                // 14009ec78  48f7d1  NOT RCX
                {
                    let v = !(r[1]);
                    r[1] = v;
                }
                // 14009ec7b  488b15f6243200  MOV RDX,qword ptr [0x1403c1178]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                }
                // 14009ec82  4c8b2577243200  MOV R12,qword ptr [0x1403c1100]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(0x1403c1100u64));
                }
                // 14009ec89  4833c2  XOR RAX,RDX
                {
                    let v = r[0] ^ r[2];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ec8c  4c8b2d7d243200  MOV R13,qword ptr [0x1403c1110]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 14009ec93  4c8b3dc6243200  MOV R15,qword ptr [0x1403c1160]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(0x1403c1160u64));
                }
                // 14009ec9a  4d33e5  XOR R12,R13
                {
                    let v = r[12] ^ r[13];
                    f.logic(v, 64);
                    r[12] = v;
                }
                // 14009ec9d  488b5c2450  MOV RBX,qword ptr [RSP + 0x50]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64));
                }
                // 14009eca2  4c0be1  OR R12,RCX
                {
                    let v = r[12] | r[1];
                    f.logic(v, 64);
                    r[12] = v;
                }
                // 14009eca5  4889442420  MOV qword ptr [RSP + 0x20],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[0]);
                }
                // 14009ecaa  488b0557243200  MOV RAX,qword ptr [0x1403c1108]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                }
                // 14009ecb1  480b442470  OR RAX,qword ptr [RSP + 0x70]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ecb6  488b7c2420  MOV RDI,qword ptr [RSP + 0x20]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                }
                // 14009ecbb  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009ecbe  4c23e0  AND R12,RAX
                {
                    let v = r[12] & r[0];
                    f.logic(v, 64);
                    r[12] = v;
                }
                pc = 0x14009ecc1;
            }
            0x14009ecc1 => {
                // 14009ecc1  4c0fbcd6  BSF R10,RSI
                {
                    let v = r[6];
                    f.z = v == 0;
                    if v != 0 {
                        r[10] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009ecc5  488d0d3413f6ff  LEA RCX,[0x140000000]
                {
                    r[1] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009eccc  4963c2  MOVSXD RAX,R10D
                {
                    r[0] = ((r[10] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009eccf  44891542062600  MOV dword ptr [0x1402ff318],R10D
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[10] & 0xffffffffu64);
                }
                // 14009ecd6  4c8b8cc1f0e00e00  MOV R9,qword ptr [RCX + RAX*0x8 + 0xee0f0]
                {
                    r[9] = m.read::<64>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xee0f0u64),
                    );
                }
                // 14009ecde  4c23ca  AND R9,RDX
                {
                    let v = r[9] & r[2];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009ece1  4c0faf8cc1f0000f00  IMUL R9,qword ptr [RCX + RAX*0x8 + 0xf00f0]
                {
                    r[9] = (r[9]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xf00f0u64),
                        ),
                    );
                }
                // 14009ecea  8b8c8100090f00  MOV ECX,dword ptr [RCX + RAX*0x4 + 0xf0900]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0900u64),
                    )) & 0xffff_ffff;
                }
                // 14009ecf1  49d3e9  SHR R9,CL
                {
                    let v = r[9];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 14009ecf4  488d0d0513f6ff  LEA RCX,[0x140000000]
                {
                    r[1] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009ecfb  488b94c1f0de0e00  MOV RDX,qword ptr [RCX + RAX*0x8 + 0xedef0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xedef0u64),
                    );
                }
                // 14009ed03  4c8b84c1f0040f00  MOV R8,qword ptr [RCX + RAX*0x8 + 0xf04f0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf04f0u64),
                    );
                }
                // 14009ed0b  4823d7  AND RDX,RDI
                {
                    let v = r[2] & r[7];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 14009ed0e  480faf94c1f0fe0e00  IMUL RDX,qword ptr [RCX + RAX*0x8 + 0xefef0]
                {
                    r[2] = (r[2]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xefef0u64),
                        ),
                    );
                }
                // 14009ed17  8b8c8100080f00  MOV ECX,dword ptr [RCX + RAX*0x4 + 0xf0800]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0800u64),
                    )) & 0xffff_ffff;
                }
                // 14009ed1e  48d3ea  SHR RDX,CL
                {
                    let v = r[2];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 14009ed21  488d0dd812f6ff  LEA RCX,[0x140000000]
                {
                    r[1] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009ed28  488b84c1f0020f00  MOV RAX,qword ptr [RCX + RAX*0x8 + 0xf02f0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf02f0u64),
                    );
                }
                // 14009ed30  4b8b0cc8  MOV RCX,qword ptr [R8 + R9*0x8]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[9].wrapping_mul(0x8u64)),
                    );
                }
                // 14009ed34  480b0cd0  OR RCX,qword ptr [RAX + RDX*0x8]
                {
                    let v = r[1]
                        | m.read::<64>(
                            0u64.wrapping_add(r[0])
                                .wrapping_add(r[2].wrapping_mul(0x8u64)),
                        );
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009ed38  8d4511  LEA EAX,[RBP + 0x11]
                {
                    r[0] = (0u64.wrapping_add(r[5]).wrapping_add(0x11u64)) & 0xffff_ffff;
                }
                // 14009ed3b  4c0bf1  OR R14,RCX
                {
                    let v = r[14] | r[1];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 14009ed3e  4885cb  TEST RBX,RCX
                {
                    let v = r[3] & r[1];
                    f.logic(v, 64);
                }
                // 14009ed41  0f44c5  CMOVZ EAX,EBP
                {
                    if f.z {
                        r[0] = (r[5] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009ed44  4180e238  AND R10B,0x38
                {
                    let v = (r[10] & 0xffu64 & 0x38u64) & 0xffu64;
                    f.logic(v, 8);
                    r[10] = (r[10] & !0xffu64) | ((v) & 0xffu64);
                }
                // 14009ed48  8be8  MOV EBP,EAX
                {
                    r[5] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009ed4a  498bc4  MOV RAX,R12
                {
                    r[0] = r[12];
                }
                // 14009ed4d  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ed50  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009ed55  69c8e3ffe4ff  IMUL ECX,EAX,-0x1b001d
                {
                    r[1] =
                        ((r[0] & 0xffffffffu64).wrapping_mul(0xffffffffffe4ffe3u64)) & 0xffff_ffff;
                }
                // 14009ed5b  4403d9  ADD R11D,ECX
                {
                    let v = f.arithmetic(r[11] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009ed5e  4180fa08  CMP R10B,0x8
                {
                    f.arithmetic(r[10] & 0xffu64, 0x8u64, 8, true);
                }
                // 14009ed62  7515  JNZ 0x14009ed79
                pc = if !f.z { 0x14009ed79 } else { 0x14009ed64 };
                continue;
            }
            0x14009ed64 => {
                // 14009ed64  498bc5  MOV RAX,R13
                {
                    r[0] = r[13];
                }
                // 14009ed67  490bc7  OR RAX,R15
                {
                    let v = r[0] | r[15];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ed6a  0fb7c0  MOVZX EAX,AX
                {
                    r[0] = (r[0] & 0xffffu64) & 0xffff_ffff;
                }
                // 14009ed6d  4885c0  TEST RAX,RAX
                {
                    let v = r[0];
                    f.logic(v, 64);
                }
                // 14009ed70  7407  JZ 0x14009ed79
                pc = if f.z { 0x14009ed79 } else { 0x14009ed72 };
                continue;
            }
            0x14009ed72 => {
                // 14009ed72  4181eb2c010000  SUB R11D,0x12c
                {
                    let v = f.arithmetic(r[11] & 0xffffffffu64, 0x12cu64, 32, true);
                    r[11] = (v) & 0xffff_ffff;
                }
                pc = 0x14009ed79;
            }
            0x14009ed79 => {
                // 14009ed79  488b15f8233200  MOV RDX,qword ptr [0x1403c1178]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                }
                // 14009ed80  488d46ff  LEA RAX,[RSI + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[6]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009ed84  4823f0  AND RSI,RAX
                {
                    let v = r[6] & r[0];
                    f.logic(v, 64);
                    r[6] = v;
                }
                // 14009ed87  0f8534ffffff  JNZ 0x14009ecc1
                pc = if !f.z { 0x14009ecc1 } else { 0x14009ed8d };
                continue;
            }
            0x14009ed8d => {
                // 14009ed8d  8b5c2430  MOV EBX,dword ptr [RSP + 0x30]
                {
                    r[3] =
                        (m.read::<32>(0u64.wrapping_add(r[4]).wrapping_add(0x30u64))) & 0xffff_ffff;
                }
                // 14009ed91  33ff  XOR EDI,EDI
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 14009ed93  4c8b7c2428  MOV R15,qword ptr [RSP + 0x28]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64));
                }
                // 14009ed98  4c8bac2488000000  MOV R13,qword ptr [RSP + 0x88]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x88u64));
                }
                pc = 0x14009eda0;
            }
            0x14009eda0 => {
                // 14009eda0  4c0b742460  OR R14,qword ptr [RSP + 0x60]
                {
                    let v = r[14] | m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64));
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 14009eda5  4c0b7c2450  OR R15,qword ptr [RSP + 0x50]
                {
                    let v = r[15] | m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64));
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009edaa  498bc6  MOV RAX,R14
                {
                    r[0] = r[14];
                }
                // 14009edad  4c8b156c233200  MOV R10,qword ptr [0x1403c1120]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1120u64));
                }
                // 14009edb4  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009edb7  488b15b6233200  MOV RDX,qword ptr [0x1403c1174]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1174u64));
                }
                // 14009edbe  4c8be0  MOV R12,RAX
                {
                    r[12] = r[0];
                }
                // 14009edc1  4c23a42490000000  AND R12,qword ptr [RSP + 0x90]
                {
                    let v = r[12] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x90u64));
                    f.logic(v, 64);
                    r[12] = v;
                }
                // 14009edc9  498bf7  MOV RSI,R15
                {
                    r[6] = r[15];
                }
                // 14009edcc  48630d9d233200  MOVSXD RCX,dword ptr [0x1403c1170]
                {
                    r[1] = ((m.read::<32>(0u64.wrapping_add(0x1403c1170u64))) as i32 as i64) as u64;
                }
                // 14009edd3  48f7d6  NOT RSI
                {
                    let v = !(r[6]);
                    r[6] = v;
                }
                // 14009edd6  4889442450  MOV qword ptr [RSP + 0x50],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64), r[0]);
                }
                // 14009eddb  488bc6  MOV RAX,RSI
                {
                    r[0] = r[6];
                }
                // 14009edde  4823842480000000  AND RAX,qword ptr [RSP + 0x80]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x80u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ede6  4c897c2428  MOV qword ptr [RSP + 0x28],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[15]);
                }
                // 14009edeb  4c89742438  MOV qword ptr [RSP + 0x38],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64), r[14]);
                }
                // 14009edf0  4889742460  MOV qword ptr [RSP + 0x60],RSI
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64), r[6]);
                }
                // 14009edf5  4889442420  MOV qword ptr [RSP + 0x20],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[0]);
                }
                // 14009edfa  4d85d2  TEST R10,R10
                {
                    let v = r[10];
                    f.logic(v, 64);
                }
                // 14009edfd  0f84d2000000  JZ 0x14009eed5
                pc = if f.z { 0x14009eed5 } else { 0x14009ee03 };
                continue;
            }
            0x14009ee03 => {
                // 14009ee03  4c8bbc2490000000  MOV R15,qword ptr [RSP + 0x90]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x90u64));
                }
                // 14009ee0b  488d3dee11f6ff  LEA RDI,[0x140000000]
                {
                    r[7] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009ee12  4863c2  MOVSXD RAX,EDX
                {
                    r[0] = ((r[2] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009ee15  4c8db7b0d71000  LEA R14,[RDI + 0x10d7b0]
                {
                    r[14] = 0u64.wrapping_add(r[7]).wrapping_add(0x10d7b0u64);
                }
                // 14009ee1c  48c1e006  SHL RAX,0x6
                {
                    let v = r[0];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009ee20  488db7b0c71000  LEA RSI,[RDI + 0x10c7b0]
                {
                    r[6] = 0u64.wrapping_add(r[7]).wrapping_add(0x10c7b0u64);
                }
                // 14009ee27  4c03f0  ADD R14,RAX
                {
                    let v = f.arithmetic(r[14], r[0], 64, false);
                    r[14] = v;
                }
                // 14009ee2a  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009ee2e  4803f1  ADD RSI,RCX
                {
                    let v = f.arithmetic(r[6], r[1], 64, false);
                    r[6] = v;
                }
                pc = 0x14009ee31;
            }
            0x14009ee31 => {
                // 14009ee31  490fbcc2  BSF RAX,R10
                {
                    let v = r[10];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009ee35  418d8b2e004500  LEA ECX,[R11 + 0x45002e]
                {
                    r[1] = (0u64.wrapping_add(r[11]).wrapping_add(0x45002eu64)) & 0xffff_ffff;
                }
                // 14009ee3c  4c63c8  MOVSXD R9,EAX
                {
                    r[9] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009ee3f  8905d3042600  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009ee45  430fbe040e  MOVSX EAX,byte ptr [R14 + R9*0x1]
                {
                    r[0] = (((m.read::<8>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[9].wrapping_mul(0x1u64)),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 14009ee4a  4e8b84cff0fa0e00  MOV R8,qword ptr [RDI + R9*0x8 + 0xefaf0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0xefaf0u64),
                    );
                }
                // 14009ee52  03d8  ADD EBX,EAX
                {
                    let v = f.arithmetic(r[3] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[3] = (v) & 0xffff_ffff;
                }
                // 14009ee54  420fbe040e  MOVSX EAX,byte ptr [RSI + R9*0x1]
                {
                    r[0] = (((m.read::<8>(
                        0u64.wrapping_add(r[6])
                            .wrapping_add(r[9].wrapping_mul(0x1u64)),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 14009ee59  4a8b94cfe0a81000  MOV RDX,qword ptr [RDI + R9*0x8 + 0x10a8e0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0x10a8e0u64),
                    );
                }
                // 14009ee61  2be8  SUB EBP,EAX
                {
                    let v = f.arithmetic(r[5] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[5] = (v) & 0xffff_ffff;
                }
                // 14009ee63  49c1e104  SHL R9,0x4
                {
                    let v = r[9];
                    let n = 4u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 14009ee67  498bc2  MOV RAX,R10
                {
                    r[0] = r[10];
                }
                // 14009ee6a  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 14009ee6d  49234528  AND RAX,qword ptr [R13 + 0x28]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x28u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ee71  4985c2  TEST R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                }
                // 14009ee74  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009ee77  410f45cb  CMOVNZ ECX,R11D
                {
                    if !f.z {
                        r[1] = (r[11] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009ee7b  4923c4  AND RAX,R12
                {
                    let v = r[0] & r[12];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ee7e  480fafc2  IMUL RAX,RDX
                {
                    r[0] = (r[0]).wrapping_mul(r[2]);
                }
                // 14009ee82  4d23c7  AND R8,R15
                {
                    let v = r[8] & r[15];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009ee85  448bd9  MOV R11D,ECX
                {
                    r[11] = (r[1] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009ee88  4c0fafc2  IMUL R8,RDX
                {
                    r[8] = (r[8]).wrapping_mul(r[2]);
                }
                // 14009ee8c  48c1e83c  SHR RAX,0x3c
                {
                    let v = r[0];
                    let n = 60u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009ee90  4903c1  ADD RAX,R9
                {
                    let v = f.arithmetic(r[0], r[9], 64, false);
                    r[0] = v;
                }
                // 14009ee93  49c1e83c  SHR R8,0x3c
                {
                    let v = r[8];
                    let n = 60u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 14009ee97  4d03c1  ADD R8,R9
                {
                    let v = f.arithmetic(r[8], r[9], 64, false);
                    r[8] = v;
                }
                // 14009ee9a  44039c87e0aa1000  ADD R11D,dword ptr [RDI + RAX*0x4 + 0x10aae0]
                {
                    let v = f.arithmetic(
                        r[11] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[7])
                                .wrapping_add(r[0].wrapping_mul(0x4u64))
                                .wrapping_add(0x10aae0u64),
                        ),
                        32,
                        false,
                    );
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009eea2  498d42ff  LEA RAX,[R10 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[10]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009eea6  46039c87e0981000  ADD R11D,dword ptr [RDI + R8*0x4 + 0x1098e0]
                {
                    let v = f.arithmetic(
                        r[11] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[7])
                                .wrapping_add(r[8].wrapping_mul(0x4u64))
                                .wrapping_add(0x1098e0u64),
                        ),
                        32,
                        false,
                    );
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009eeae  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009eeb1  0f857affffff  JNZ 0x14009ee31
                pc = if !f.z { 0x14009ee31 } else { 0x14009eeb7 };
                continue;
            }
            0x14009eeb7 => {
                // 14009eeb7  4c8b742438  MOV R14,qword ptr [RSP + 0x38]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64));
                }
                // 14009eebc  33ff  XOR EDI,EDI
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 14009eebe  4c8b7c2428  MOV R15,qword ptr [RSP + 0x28]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64));
                }
                // 14009eec3  488b742460  MOV RSI,qword ptr [RSP + 0x60]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64));
                }
                // 14009eec8  8b0da2223200  MOV ECX,dword ptr [0x1403c1170]
                {
                    r[1] = (m.read::<32>(0u64.wrapping_add(0x1403c1170u64))) & 0xffff_ffff;
                }
                // 14009eece  488b159f223200  MOV RDX,qword ptr [0x1403c1174]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1174u64));
                }
                pc = 0x14009eed5;
            }
            0x14009eed5 => {
                // 14009eed5  4c8b154c223200  MOV R10,qword ptr [0x1403c1128]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1128u64));
                }
                // 14009eedc  4d85d2  TEST R10,R10
                {
                    let v = r[10];
                    f.logic(v, 64);
                }
                // 14009eedf  0f84d2000000  JZ 0x14009efb7
                pc = if f.z { 0x14009efb7 } else { 0x14009eee5 };
                continue;
            }
            0x14009eee5 => {
                // 14009eee5  4c8b642420  MOV R12,qword ptr [RSP + 0x20]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                }
                // 14009eeea  4c8d3d0f11f6ff  LEA R15,[0x140000000]
                {
                    r[15] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009eef1  488bb42480000000  MOV RSI,qword ptr [RSP + 0x80]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x80u64));
                }
                // 14009eef9  498dbfb0d71000  LEA RDI,[R15 + 0x10d7b0]
                {
                    r[7] = 0u64.wrapping_add(r[15]).wrapping_add(0x10d7b0u64);
                }
                // 14009ef00  4863c1  MOVSXD RAX,ECX
                {
                    r[0] = ((r[1] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009ef03  4d8db7b0c71000  LEA R14,[R15 + 0x10c7b0]
                {
                    r[14] = 0u64.wrapping_add(r[15]).wrapping_add(0x10c7b0u64);
                }
                // 14009ef0a  48c1e006  SHL RAX,0x6
                {
                    let v = r[0];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009ef0e  4863ca  MOVSXD RCX,EDX
                {
                    r[1] = ((r[2] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009ef11  4803f8  ADD RDI,RAX
                {
                    let v = f.arithmetic(r[7], r[0], 64, false);
                    r[7] = v;
                }
                // 14009ef14  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009ef18  4c03f1  ADD R14,RCX
                {
                    let v = f.arithmetic(r[14], r[1], 64, false);
                    r[14] = v;
                }
                // 14009ef1b  0f1f440000  NOP dword ptr [RAX + RAX*0x1]
                pc = 0x14009ef20;
            }
            0x14009ef20 => {
                // 14009ef20  490fbcc2  BSF RAX,R10
                {
                    let v = r[10];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009ef24  418d8bd2ffbaff  LEA ECX,[R11 + -0x45002e]
                {
                    r[1] = (0u64.wrapping_add(r[11]).wrapping_add(0xffffffffffbaffd2u64))
                        & 0xffff_ffff;
                }
                // 14009ef2b  4c63c8  MOVSXD R9,EAX
                {
                    r[9] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009ef2e  8905e4032600  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009ef34  420fbe040f  MOVSX EAX,byte ptr [RDI + R9*0x1]
                {
                    r[0] = (((m.read::<8>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[9].wrapping_mul(0x1u64)),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 14009ef39  4f8b84cff0fa0e00  MOV R8,qword ptr [R15 + R9*0x8 + 0xefaf0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0xefaf0u64),
                    );
                }
                // 14009ef41  03e8  ADD EBP,EAX
                {
                    let v = f.arithmetic(r[5] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[5] = (v) & 0xffff_ffff;
                }
                // 14009ef43  430fbe040e  MOVSX EAX,byte ptr [R14 + R9*0x1]
                {
                    r[0] = (((m.read::<8>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[9].wrapping_mul(0x1u64)),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 14009ef48  4b8b94cfe0861000  MOV RDX,qword ptr [R15 + R9*0x8 + 0x1086e0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0x1086e0u64),
                    );
                }
                // 14009ef50  2bd8  SUB EBX,EAX
                {
                    let v = f.arithmetic(r[3] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[3] = (v) & 0xffff_ffff;
                }
                // 14009ef52  49c1e104  SHL R9,0x4
                {
                    let v = r[9];
                    let n = 4u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 14009ef56  498bc2  MOV RAX,R10
                {
                    r[0] = r[10];
                }
                // 14009ef59  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 14009ef5c  4923c2  AND RAX,R10
                {
                    let v = r[0] & r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ef5f  49854520  TEST qword ptr [R13 + 0x20],RAX
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[13]).wrapping_add(0x20u64)) & r[0];
                    f.logic(v, 64);
                }
                // 14009ef63  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 14009ef66  410f45cb  CMOVNZ ECX,R11D
                {
                    if !f.z {
                        r[1] = (r[11] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 14009ef6a  4923c4  AND RAX,R12
                {
                    let v = r[0] & r[12];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009ef6d  480fafc2  IMUL RAX,RDX
                {
                    r[0] = (r[0]).wrapping_mul(r[2]);
                }
                // 14009ef71  4c23c6  AND R8,RSI
                {
                    let v = r[8] & r[6];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009ef74  448bd9  MOV R11D,ECX
                {
                    r[11] = (r[1] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009ef77  4c0fafc2  IMUL R8,RDX
                {
                    r[8] = (r[8]).wrapping_mul(r[2]);
                }
                // 14009ef7b  48c1e83c  SHR RAX,0x3c
                {
                    let v = r[0];
                    let n = 60u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009ef7f  4903c1  ADD RAX,R9
                {
                    let v = f.arithmetic(r[0], r[9], 64, false);
                    r[0] = v;
                }
                // 14009ef82  49c1e83c  SHR R8,0x3c
                {
                    let v = r[8];
                    let n = 60u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 14009ef86  4d03c1  ADD R8,R9
                {
                    let v = f.arithmetic(r[8], r[9], 64, false);
                    r[8] = v;
                }
                // 14009ef89  452b9c87e0881000  SUB R11D,dword ptr [R15 + RAX*0x4 + 0x1088e0]
                {
                    let v = f.arithmetic(
                        r[11] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[15])
                                .wrapping_add(r[0].wrapping_mul(0x4u64))
                                .wrapping_add(0x1088e0u64),
                        ),
                        32,
                        true,
                    );
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009ef91  498d42ff  LEA RAX,[R10 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[10]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009ef95  472b9c87e0761000  SUB R11D,dword ptr [R15 + R8*0x4 + 0x1076e0]
                {
                    let v = f.arithmetic(
                        r[11] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[15])
                                .wrapping_add(r[8].wrapping_mul(0x4u64))
                                .wrapping_add(0x1076e0u64),
                        ),
                        32,
                        true,
                    );
                    r[11] = (v) & 0xffff_ffff;
                }
                // 14009ef9d  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009efa0  0f857affffff  JNZ 0x14009ef20
                pc = if !f.z { 0x14009ef20 } else { 0x14009efa6 };
                continue;
            }
            0x14009efa6 => {
                // 14009efa6  4c8b742438  MOV R14,qword ptr [RSP + 0x38]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x38u64));
                }
                // 14009efab  33ff  XOR EDI,EDI
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 14009efad  4c8b7c2428  MOV R15,qword ptr [RSP + 0x28]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64));
                }
                // 14009efb2  488b742460  MOV RSI,qword ptr [RSP + 0x60]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x60u64));
                }
                pc = 0x14009efb7;
            }
            0x14009efb7 => {
                // 14009efb7  488b542478  MOV RDX,qword ptr [RSP + 0x78]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x78u64));
                }
                // 14009efbc  4c8b253d213200  MOV R12,qword ptr [0x1403c1100]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(0x1403c1100u64));
                }
                // 14009efc3  4c8b442458  MOV R8,qword ptr [RSP + 0x58]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64));
                }
                // 14009efc8  4d8bcc  MOV R9,R12
                {
                    r[9] = r[12];
                }
                // 14009efcb  488b4c2448  MOV RCX,qword ptr [RSP + 0x48]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64));
                }
                // 14009efd0  4c23ce  AND R9,RSI
                {
                    let v = r[9] & r[6];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009efd3  488b052e213200  MOV RAX,qword ptr [0x1403c1108]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                }
                // 14009efda  4d23ce  AND R9,R14
                {
                    let v = r[9] & r[14];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009efdd  4c8bd0  MOV R10,RAX
                {
                    r[10] = r[0];
                }
                // 14009efe0  4c893a  MOV qword ptr [RDX],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[2]), r[15]);
                }
                // 14009efe3  4c23542450  AND R10,qword ptr [RSP + 0x50]
                {
                    let v = r[10] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64));
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009efe8  4d23d7  AND R10,R15
                {
                    let v = r[10] & r[15];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009efeb  4c897208  MOV qword ptr [RDX + 0x8],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[2]).wrapping_add(0x8u64), r[14]);
                }
                // 14009efef  4923c2  AND RAX,R10
                {
                    let v = r[0] & r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009eff2  490bc0  OR RAX,R8
                {
                    let v = r[0] | r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009eff5  49f7d0  NOT R8
                {
                    let v = !(r[8]);
                    r[8] = v;
                }
                // 14009eff8  48894220  MOV qword ptr [RDX + 0x20],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[2]).wrapping_add(0x20u64), r[0]);
                }
                // 14009effc  498bc4  MOV RAX,R12
                {
                    r[0] = r[12];
                }
                // 14009efff  4923c1  AND RAX,R9
                {
                    let v = r[0] & r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f002  4c89442458  MOV qword ptr [RSP + 0x58],R8
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64), r[8]);
                }
                // 14009f007  480bc1  OR RAX,RCX
                {
                    let v = r[0] | r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f00a  48f7d1  NOT RCX
                {
                    let v = !(r[1]);
                    r[1] = v;
                }
                // 14009f00d  48894228  MOV qword ptr [RDX + 0x28],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[2]).wrapping_add(0x28u64), r[0]);
                }
                // 14009f011  488b0508213200  MOV RAX,qword ptr [0x1403c1120]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1120u64));
                }
                // 14009f018  480b0511213200  OR RAX,qword ptr [0x1403c1130]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1130u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f01f  480b051a213200  OR RAX,qword ptr [0x1403c1140]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1140u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f026  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f029  488b1548213200  MOV RDX,qword ptr [0x1403c1178]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                }
                // 14009f030  4823442470  AND RAX,qword ptr [RSP + 0x70]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f035  48f7d2  NOT RDX
                {
                    let v = !(r[2]);
                    r[2] = v;
                }
                // 14009f038  48894c2448  MOV qword ptr [RSP + 0x48],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64), r[1]);
                }
                // 14009f03d  f3480fb8c8  POPCNT RCX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[1] = u64::from(v.count_ones());
                }
                // 14009f042  488b05df203200  MOV RAX,qword ptr [0x1403c1128]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1128u64));
                }
                // 14009f049  480b05e8203200  OR RAX,qword ptr [0x1403c1138]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1138u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f050  480b05f1203200  OR RAX,qword ptr [0x1403c1148]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f057  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f05a  4823442468  AND RAX,qword ptr [RSP + 0x68]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x68u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f05f  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009f064  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f066  488b442468  MOV RAX,qword ptr [RSP + 0x68]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x68u64));
                }
                // 14009f06b  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009f06e  4469c116001600  IMUL R8D,ECX,0x160016
                {
                    r[8] = ((r[1] & 0xffffffffu64).wrapping_mul(0x160016u64)) & 0xffff_ffff;
                }
                // 14009f075  4823058c203200  AND RAX,qword ptr [0x1403c1108]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f07c  4923c7  AND RAX,R15
                {
                    let v = r[0] & r[15];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f07f  f3480fb8c8  POPCNT RCX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[1] = u64::from(v.count_ones());
                }
                // 14009f084  488b442470  MOV RAX,qword ptr [RSP + 0x70]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x70u64));
                }
                // 14009f089  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 14009f08c  4923c4  AND RAX,R12
                {
                    let v = r[0] & r[12];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f08f  4923c6  AND RAX,R14
                {
                    let v = r[0] & r[14];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f092  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009f097  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f099  69c13c001e00  IMUL EAX,ECX,0x1e003c
                {
                    r[0] = ((r[1] & 0xffffffffu64).wrapping_mul(0x1e003cu64)) & 0xffff_ffff;
                }
                // 14009f09f  4403c0  ADD R8D,EAX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009f0a2  488b056f203200  MOV RAX,qword ptr [0x1403c1118]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 14009f0a9  48c1e808  SHR RAX,0x8
                {
                    let v = r[0];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009f0ad  4823c2  AND RAX,RDX
                {
                    let v = r[0] & r[2];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f0b0  f3480fb8c8  POPCNT RCX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[1] = u64::from(v.count_ones());
                }
                // 14009f0b5  488b0554203200  MOV RAX,qword ptr [0x1403c1110]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 14009f0bc  48c1e008  SHL RAX,0x8
                {
                    let v = r[0];
                    let n = 8u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 14009f0c0  4823c2  AND RAX,RDX
                {
                    let v = r[0] & r[2];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f0c3  488b542458  MOV RDX,qword ptr [RSP + 0x58]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x58u64));
                }
                // 14009f0c8  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009f0cd  2bc1  SUB EAX,ECX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f0cf  69c890002000  IMUL ECX,EAX,0x200090
                {
                    r[1] = ((r[0] & 0xffffffffu64).wrapping_mul(0x200090u64)) & 0xffff_ffff;
                }
                // 14009f0d5  488b052c203200  MOV RAX,qword ptr [0x1403c1108]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                }
                // 14009f0dc  4823c2  AND RAX,RDX
                {
                    let v = r[0] & r[2];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f0df  4923d2  AND RDX,R10
                {
                    let v = r[2] & r[10];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 14009f0e2  4923c6  AND RAX,R14
                {
                    let v = r[0] & r[14];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f0e5  4403c1  ADD R8D,ECX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009f0e8  f3480fb8c8  POPCNT RCX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[1] = u64::from(v.count_ones());
                }
                // 14009f0ed  498bc4  MOV RAX,R12
                {
                    r[0] = r[12];
                }
                // 14009f0f0  4823442448  AND RAX,qword ptr [RSP + 0x48]
                {
                    let v = r[0] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f0f5  4923c7  AND RAX,R15
                {
                    let v = r[0] & r[15];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f0f8  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009f0fd  2bc1  SUB EAX,ECX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f0ff  69c80c000c00  IMUL ECX,EAX,0xc000c
                {
                    r[1] = ((r[0] & 0xffffffffu64).wrapping_mul(0xc000cu64)) & 0xffff_ffff;
                }
                // 14009f105  488b442448  MOV RAX,qword ptr [RSP + 0x48]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x48u64));
                }
                // 14009f10a  4923c1  AND RAX,R9
                {
                    let v = r[0] & r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f10d  4403c1  ADD R8D,ECX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009f110  f3480fb8ca  POPCNT RCX,RDX
                {
                    let v = r[2];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[1] = u64::from(v.count_ones());
                }
                // 14009f115  f3480fb8c0  POPCNT RAX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009f11a  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f11c  488b052d203200  MOV RAX,qword ptr [0x1403c1150]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c1150u64));
                }
                // 14009f123  4469c9b4007800  IMUL R9D,ECX,0x7800b4
                {
                    r[9] = ((r[1] & 0xffffffffu64).wrapping_mul(0x7800b4u64)) & 0xffff_ffff;
                }
                // 14009f12a  4503cb  ADD R9D,R11D
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[11] & 0xffffffffu64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 14009f12d  4503c8  ADD R9D,R8D
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 14009f130  480b0521203200  OR RAX,qword ptr [0x1403c1158]
                {
                    let v = r[0] | m.read::<64>(0u64.wrapping_add(0x1403c1158u64));
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f137  7428  JZ 0x14009f161
                pc = if f.z { 0x14009f161 } else { 0x14009f139 };
                continue;
            }
            0x14009f139 => {
                // 14009f139  4863cd  MOVSXD RCX,EBP
                {
                    r[1] = ((r[5] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f13c  488d2dbd0ef6ff  LEA RBP,[0x140000000]
                {
                    r[5] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009f143  4863c3  MOVSXD RAX,EBX
                {
                    r[0] = ((r[3] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f146  8b948510f11e00  MOV EDX,dword ptr [RBP + RAX*0x4 + 0x1ef110]
                {
                    r[2] = (m.read::<32>(
                        0u64.wrapping_add(r[5])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0x1ef110u64),
                    )) & 0xffff_ffff;
                }
                // 14009f14d  2b948d10f11e00  SUB EDX,dword ptr [RBP + RCX*0x4 + 0x1ef110]
                {
                    let v = f.arithmetic(
                        r[2] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[5])
                                .wrapping_add(r[1].wrapping_mul(0x4u64))
                                .wrapping_add(0x1ef110u64),
                        ),
                        32,
                        true,
                    );
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f154  412b5534  SUB EDX,dword ptr [R13 + 0x34]
                {
                    let v = f.arithmetic(
                        r[2] & 0xffffffffu64,
                        m.read::<32>(0u64.wrapping_add(r[13]).wrapping_add(0x34u64)),
                        32,
                        true,
                    );
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f158  41035538  ADD EDX,dword ptr [R13 + 0x38]
                {
                    let v = f.arithmetic(
                        r[2] & 0xffffffffu64,
                        m.read::<32>(0u64.wrapping_add(r[13]).wrapping_add(0x38u64)),
                        32,
                        false,
                    );
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f15c  4403ca  ADD R9D,EDX
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 14009f15f  eb07  JMP 0x14009f168
                pc = 0x14009f168;
                continue;
            }
            0x14009f161 => {
                // 14009f161  488d2d980ef6ff  LEA RBP,[0x140000000]
                {
                    r[5] = 0u64.wrapping_add(0x140000000u64);
                }
                pc = 0x14009f168;
            }
            0x14009f168 => {
                // 14009f168  488b1da11f3200  MOV RBX,qword ptr [0x1403c1110]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 14009f16f  48b8000000ffffffff00  MOV RAX,0xffffffff000000
                {
                    r[0] = 0xffffffff000000u64;
                }
                // 14009f179  4c8b1d881f3200  MOV R11,qword ptr [0x1403c1108]
                {
                    r[11] = m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                }
                // 14009f180  4c8bd3  MOV R10,RBX
                {
                    r[10] = r[3];
                }
                // 14009f183  4d235500  AND R10,qword ptr [R13]
                {
                    let v = r[10] & m.read::<64>(0u64.wrapping_add(r[13]));
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009f187  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009f18a  0f848d000000  JZ 0x14009f21d
                pc = if f.z { 0x14009f21d } else { 0x14009f190 };
                continue;
            }
            0x14009f190 => {
                // 14009f190  490fbcd2  BSF RDX,R10
                {
                    let v = r[10];
                    f.z = v == 0;
                    if v != 0 {
                        r[2] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f194  4863c2  MOVSXD RAX,EDX
                {
                    r[0] = ((r[2] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f197  89157b012600  MOV dword ptr [0x1402ff318],EDX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[2] & 0xffffffffu64);
                }
                // 14009f19d  4c8b84c580440f00  MOV R8,qword ptr [RBP + RAX*0x8 + 0xf4480]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[5])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4480u64),
                    );
                }
                // 14009f1a5  4d85c4  TEST R12,R8
                {
                    let v = r[12] & r[8];
                    f.logic(v, 64);
                }
                // 14009f1a8  7512  JNZ 0x14009f1bc
                pc = if !f.z { 0x14009f1bc } else { 0x14009f1aa };
                continue;
            }
            0x14009f1aa => {
                // 14009f1aa  8d42e8  LEA EAX,[RDX + -0x18]
                {
                    r[0] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffe8u64)) & 0xffff_ffff;
                }
                // 14009f1ad  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f1b0  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f1b4  44038c29e84a1000  ADD R9D,dword ptr [RCX + RBP*0x1 + 0x104ae8]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[5].wrapping_mul(0x1u64))
                                .wrapping_add(0x104ae8u64),
                        ),
                        32,
                        false,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f1bc;
            }
            0x14009f1bc => {
                // 14009f1bc  4d85c3  TEST R11,R8
                {
                    let v = r[11] & r[8];
                    f.logic(v, 64);
                }
                // 14009f1bf  7512  JNZ 0x14009f1d3
                pc = if !f.z { 0x14009f1d3 } else { 0x14009f1c1 };
                continue;
            }
            0x14009f1c1 => {
                // 14009f1c1  8d42e8  LEA EAX,[RDX + -0x18]
                {
                    r[0] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffe8u64)) & 0xffff_ffff;
                }
                // 14009f1c4  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f1c7  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f1cb  44038c29ec4a1000  ADD R9D,dword ptr [RCX + RBP*0x1 + 0x104aec]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[5].wrapping_mul(0x1u64))
                                .wrapping_add(0x104aecu64),
                        ),
                        32,
                        false,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f1d3;
            }
            0x14009f1d3 => {
                // 14009f1d3  8d4208  LEA EAX,[RDX + 0x8]
                {
                    r[0] = (0u64.wrapping_add(r[2]).wrapping_add(0x8u64)) & 0xffff_ffff;
                }
                // 14009f1d6  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f1d9  4883bccd000e3c0000  CMP qword ptr [RBP + RCX*0x8 + 0x3c0e00],0x0
                {
                    f.arithmetic(
                        m.read::<64>(
                            0u64.wrapping_add(r[5])
                                .wrapping_add(r[1].wrapping_mul(0x8u64))
                                .wrapping_add(0x3c0e00u64),
                        ),
                        0x0u64,
                        64,
                        true,
                    );
                }
                // 14009f1e2  7512  JNZ 0x14009f1f6
                pc = if !f.z { 0x14009f1f6 } else { 0x14009f1e4 };
                continue;
            }
            0x14009f1e4 => {
                // 14009f1e4  8d42e8  LEA EAX,[RDX + -0x18]
                {
                    r[0] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffe8u64)) & 0xffff_ffff;
                }
                // 14009f1e7  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f1ea  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f1ee  44038c29f04a1000  ADD R9D,dword ptr [RCX + RBP*0x1 + 0x104af0]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[5].wrapping_mul(0x1u64))
                                .wrapping_add(0x104af0u64),
                        ),
                        32,
                        false,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f1f6;
            }
            0x14009f1f6 => {
                // 14009f1f6  4c23c6  AND R8,RSI
                {
                    let v = r[8] & r[6];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009f1f9  4d85c6  TEST R14,R8
                {
                    let v = r[14] & r[8];
                    f.logic(v, 64);
                }
                // 14009f1fc  7512  JNZ 0x14009f210
                pc = if !f.z { 0x14009f210 } else { 0x14009f1fe };
                continue;
            }
            0x14009f1fe => {
                // 14009f1fe  8d42e8  LEA EAX,[RDX + -0x18]
                {
                    r[0] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffe8u64)) & 0xffff_ffff;
                }
                // 14009f201  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f204  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f208  44038c29f44a1000  ADD R9D,dword ptr [RCX + RBP*0x1 + 0x104af4]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[5].wrapping_mul(0x1u64))
                                .wrapping_add(0x104af4u64),
                        ),
                        32,
                        false,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f210;
            }
            0x14009f210 => {
                // 14009f210  498d42ff  LEA RAX,[R10 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[10]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009f214  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009f217  0f8573ffffff  JNZ 0x14009f190
                pc = if !f.z { 0x14009f190 } else { 0x14009f21d };
                continue;
            }
            0x14009f21d => {
                // 14009f21d  488b2df41e3200  MOV RBP,qword ptr [0x1403c1118]
                {
                    r[5] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 14009f224  4c8d35d50df6ff  LEA R14,[0x140000000]
                {
                    r[14] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009f22b  4c8bd5  MOV R10,RBP
                {
                    r[10] = r[5];
                }
                // 14009f22e  48b800ffffffff000000  MOV RAX,0xffffffff00
                {
                    r[0] = 0xffffffff00u64;
                }
                // 14009f238  4d235500  AND R10,qword ptr [R13]
                {
                    let v = r[10] & m.read::<64>(0u64.wrapping_add(r[13]));
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009f23c  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009f23f  0f84af000000  JZ 0x14009f2f4
                pc = if f.z { 0x14009f2f4 } else { 0x14009f245 };
                continue;
            }
            0x14009f245 => {
                // 14009f245  488b5c2450  MOV RBX,qword ptr [RSP + 0x50]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x50u64));
                }
                // 14009f24a  660f1f440000  NOP word ptr [RAX + RAX*0x1]
                pc = 0x14009f250;
            }
            0x14009f250 => {
                // 14009f250  490fbcd2  BSF RDX,R10
                {
                    let v = r[10];
                    f.z = v == 0;
                    if v != 0 {
                        r[2] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f254  4863c2  MOVSXD RAX,EDX
                {
                    r[0] = ((r[2] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f257  8915bb002600  MOV dword ptr [0x1402ff318],EDX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[2] & 0xffffffffu64);
                }
                // 14009f25d  4d8b84c680420f00  MOV R8,qword ptr [R14 + RAX*0x8 + 0xf4280]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4280u64),
                    );
                }
                // 14009f265  4d85c3  TEST R11,R8
                {
                    let v = r[11] & r[8];
                    f.logic(v, 64);
                }
                // 14009f268  7516  JNZ 0x14009f280
                pc = if !f.z { 0x14009f280 } else { 0x14009f26a };
                continue;
            }
            0x14009f26a => {
                // 14009f26a  b827000000  MOV EAX,0x27
                {
                    r[0] = (0x27u64) & 0xffff_ffff;
                }
                // 14009f26f  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f271  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f274  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f278  462b8c31084b1000  SUB R9D,dword ptr [RCX + R14*0x1 + 0x104b08]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[14].wrapping_mul(0x1u64))
                                .wrapping_add(0x104b08u64),
                        ),
                        32,
                        true,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f280;
            }
            0x14009f280 => {
                // 14009f280  4d85c4  TEST R12,R8
                {
                    let v = r[12] & r[8];
                    f.logic(v, 64);
                }
                // 14009f283  7516  JNZ 0x14009f29b
                pc = if !f.z { 0x14009f29b } else { 0x14009f285 };
                continue;
            }
            0x14009f285 => {
                // 14009f285  b827000000  MOV EAX,0x27
                {
                    r[0] = (0x27u64) & 0xffff_ffff;
                }
                // 14009f28a  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f28c  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f28f  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f293  462b8c310c4b1000  SUB R9D,dword ptr [RCX + R14*0x1 + 0x104b0c]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[14].wrapping_mul(0x1u64))
                                .wrapping_add(0x104b0cu64),
                        ),
                        32,
                        true,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f29b;
            }
            0x14009f29b => {
                // 14009f29b  8d42f8  LEA EAX,[RDX + -0x8]
                {
                    r[0] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xfffffffffffffff8u64)) & 0xffff_ffff;
                }
                // 14009f29e  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f2a1  4983bcce000e3c0000  CMP qword ptr [R14 + RCX*0x8 + 0x3c0e00],0x0
                {
                    f.arithmetic(
                        m.read::<64>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[1].wrapping_mul(0x8u64))
                                .wrapping_add(0x3c0e00u64),
                        ),
                        0x0u64,
                        64,
                        true,
                    );
                }
                // 14009f2aa  7516  JNZ 0x14009f2c2
                pc = if !f.z { 0x14009f2c2 } else { 0x14009f2ac };
                continue;
            }
            0x14009f2ac => {
                // 14009f2ac  b827000000  MOV EAX,0x27
                {
                    r[0] = (0x27u64) & 0xffff_ffff;
                }
                // 14009f2b1  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f2b3  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f2b6  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f2ba  462b8c31104b1000  SUB R9D,dword ptr [RCX + R14*0x1 + 0x104b10]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[14].wrapping_mul(0x1u64))
                                .wrapping_add(0x104b10u64),
                        ),
                        32,
                        true,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f2c2;
            }
            0x14009f2c2 => {
                // 14009f2c2  4c23c3  AND R8,RBX
                {
                    let v = r[8] & r[3];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009f2c5  4d85c7  TEST R15,R8
                {
                    let v = r[15] & r[8];
                    f.logic(v, 64);
                }
                // 14009f2c8  7516  JNZ 0x14009f2e0
                pc = if !f.z { 0x14009f2e0 } else { 0x14009f2ca };
                continue;
            }
            0x14009f2ca => {
                // 14009f2ca  b827000000  MOV EAX,0x27
                {
                    r[0] = (0x27u64) & 0xffff_ffff;
                }
                // 14009f2cf  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f2d1  4863c8  MOVSXD RCX,EAX
                {
                    r[1] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f2d4  48c1e106  SHL RCX,0x6
                {
                    let v = r[1];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f2d8  462b8c31144b1000  SUB R9D,dword ptr [RCX + R14*0x1 + 0x104b14]
                {
                    let v = f.arithmetic(
                        r[9] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[1])
                                .wrapping_add(r[14].wrapping_mul(0x1u64))
                                .wrapping_add(0x104b14u64),
                        ),
                        32,
                        true,
                    );
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f2e0;
            }
            0x14009f2e0 => {
                // 14009f2e0  498d42ff  LEA RAX,[R10 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[10]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009f2e4  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 14009f2e7  0f8563ffffff  JNZ 0x14009f250
                pc = if !f.z { 0x14009f250 } else { 0x14009f2ed };
                continue;
            }
            0x14009f2ed => {
                // 14009f2ed  488b1d1c1e3200  MOV RBX,qword ptr [0x1403c1110]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                pc = 0x14009f2f4;
            }
            0x14009f2f4 => {
                // 14009f2f4  486305751e3200  MOVSXD RAX,dword ptr [0x1403c1170]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1170u64))) as i32 as i64) as u64;
                }
                // 14009f2fb  488b353e1e3200  MOV RSI,qword ptr [0x1403c1140]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1140u64));
                }
                // 14009f302  498b8cc6f0ec0e00  MOV RCX,qword ptr [R14 + RAX*0x8 + 0xeecf0]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xeecf0u64),
                    );
                }
                // 14009f30a  4823ce  AND RCX,RSI
                {
                    let v = r[1] & r[6];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009f30d  742b  JZ 0x14009f33a
                pc = if f.z { 0x14009f33a } else { 0x14009f30f };
                continue;
            }
            0x14009f30f => {
                // 14009f30f  480fbcc1  BSF RAX,RCX
                {
                    let v = r[1];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f313  48b9ffffffffff000000  MOV RCX,0xffffffffff
                {
                    r[1] = 0xffffffffffu64;
                }
                // 14009f31d  8905f5ff2500  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009f323  498b84c680440f00  MOV RAX,qword ptr [R14 + RAX*0x8 + 0xf4480]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4480u64),
                    );
                }
                // 14009f32b  4823c3  AND RAX,RBX
                {
                    let v = r[0] & r[3];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f32e  4885c1  TEST RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                }
                // 14009f331  7407  JZ 0x14009f33a
                pc = if f.z { 0x14009f33a } else { 0x14009f333 };
                continue;
            }
            0x14009f333 => {
                // 14009f333  4181e900005301  SUB R9D,0x1530000
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, 0x1530000u64, 32, true);
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f33a;
            }
            0x14009f33a => {
                // 14009f33a  486305331e3200  MOVSXD RAX,dword ptr [0x1403c1174]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 14009f341  498b8cc6f0ec0e00  MOV RCX,qword ptr [R14 + RAX*0x8 + 0xeecf0]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xeecf0u64),
                    );
                }
                // 14009f349  48230df81d3200  AND RCX,qword ptr [0x1403c1148]
                {
                    let v = r[1] & m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009f350  7424  JZ 0x14009f376
                pc = if f.z { 0x14009f376 } else { 0x14009f352 };
                continue;
            }
            0x14009f352 => {
                // 14009f352  480fbcc1  BSF RAX,RCX
                {
                    let v = r[1];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f356  8905bcff2500  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009f35c  498b84c680420f00  MOV RAX,qword ptr [R14 + RAX*0x8 + 0xf4280]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4280u64),
                    );
                }
                // 14009f364  4823c5  AND RAX,RBP
                {
                    let v = r[0] & r[5];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f367  48a9000000ff  TEST RAX,-0x1000000
                {
                    let v = r[0] & 0xffffffffff000000u64;
                    f.logic(v, 64);
                }
                // 14009f36d  7407  JZ 0x14009f376
                pc = if f.z { 0x14009f376 } else { 0x14009f36f };
                continue;
            }
            0x14009f36f => {
                // 14009f36f  4181c100005301  ADD R9D,0x1530000
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, 0x1530000u64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f376;
            }
            0x14009f376 => {
                // 14009f376  4c8b353b1e3200  MOV R14,qword ptr [0x1403c11b8]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(0x1403c11b8u64));
                }
                // 14009f37d  48b800000000fcff0000  MOV RAX,0xfffc00000000
                {
                    r[0] = 0xfffc00000000u64;
                }
                // 14009f387  4d8bfe  MOV R15,R14
                {
                    r[15] = r[14];
                }
                // 14009f38a  4c23f8  AND R15,RAX
                {
                    let v = r[15] & r[0];
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 14009f38d  0f8571010000  JNZ 0x14009f504
                pc = if !f.z { 0x14009f504 } else { 0x14009f393 };
                continue;
            }
            0x14009f393 => {
                // 14009f393  488b35c61d3200  MOV RSI,qword ptr [0x1403c1160]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1160u64));
                }
                // 14009f39a  4c8bc3  MOV R8,RBX
                {
                    r[8] = r[3];
                }
                // 14009f39d  448bd7  MOV R10D,EDI
                {
                    r[10] = (r[7] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f3a0  4d234500  AND R8,qword ptr [R13]
                {
                    let v = r[8] & m.read::<64>(0u64.wrapping_add(r[13]));
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009f3a4  0f8491000000  JZ 0x14009f43b
                pc = if f.z { 0x14009f43b } else { 0x14009f3aa };
                continue;
            }
            0x14009f3aa => {
                // 14009f3aa  4c8b1db71d3200  MOV R11,qword ptr [0x1403c1168]
                {
                    r[11] = m.read::<64>(0u64.wrapping_add(0x1403c1168u64));
                }
                // 14009f3b1  488d05c8360500  LEA RAX,[0x1400f2a80]
                {
                    r[0] = 0u64.wrapping_add(0x1400f2a80u64);
                }
                // 14009f3b8  4c8b742478  MOV R14,qword ptr [RSP + 0x78]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x78u64));
                }
                // 14009f3bd  488d1d3c0cf6ff  LEA RBX,[0x140000000]
                {
                    r[3] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009f3c4  4889442420  MOV qword ptr [RSP + 0x20],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[0]);
                }
                // 14009f3c9  0f1f8000000000  NOP dword ptr [RAX]
                pc = 0x14009f3d0;
            }
            0x14009f3d0 => {
                // 14009f3d0  490fbcd0  BSF RDX,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    if v != 0 {
                        r[2] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f3d4  4863c2  MOVSXD RAX,EDX
                {
                    r[0] = ((r[2] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f3d7  89153bff2500  MOV dword ptr [0x1402ff318],EDX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[2] & 0xffffffffu64);
                }
                // 14009f3dd  4885b4c380240f00  TEST qword ptr [RBX + RAX*0x8 + 0xf2480],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf2480u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 14009f3e5  7522  JNZ 0x14009f409
                pc = if !f.z { 0x14009f409 } else { 0x14009f3e7 };
                continue;
            }
            0x14009f3e7 => {
                // 14009f3e7  4c85a4c380440f00  TEST qword ptr [RBX + RAX*0x8 + 0xf4480],R12
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4480u64),
                    ) & r[12];
                    f.logic(v, 64);
                }
                // 14009f3ef  752c  JNZ 0x14009f41d
                pc = if !f.z { 0x14009f41d } else { 0x14009f3f1 };
                continue;
            }
            0x14009f3f1 => {
                // 14009f3f1  41837e7000  CMP dword ptr [R14 + 0x70],0x0
                {
                    f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[14]).wrapping_add(0x70u64)),
                        0x0u64,
                        32,
                        true,
                    );
                }
                // 14009f3f6  488d0d83380500  LEA RCX,[0x1400f2c80]
                {
                    r[1] = 0u64.wrapping_add(0x1400f2c80u64);
                }
                // 14009f3fd  480f454c2420  CMOVNZ RCX,qword ptr [RSP + 0x20]
                {
                    if !f.z {
                        r[1] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                    }
                }
                // 14009f403  4c851cc1  TEST qword ptr [RCX + RAX*0x8],R11
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x8u64)),
                    ) & r[11];
                    f.logic(v, 64);
                }
                // 14009f407  7414  JZ 0x14009f41d
                pc = if f.z { 0x14009f41d } else { 0x14009f409 };
                continue;
            }
            0x14009f409 => {
                // 14009f409  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f40b  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 14009f40c  83e207  AND EDX,0x7
                {
                    let v = (r[2] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f40f  03c2  ADD EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f411  c1f803  SAR EAX,0x3
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 3u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f414  412b4670  SUB EAX,dword ptr [R14 + 0x70]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(0u64.wrapping_add(r[14]).wrapping_add(0x70u64)),
                        32,
                        true,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f418  3bf8  CMP EDI,EAX
                {
                    f.arithmetic(r[7] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                }
                // 14009f41a  0f4cf8  CMOVL EDI,EAX
                {
                    if f.s != f.o {
                        r[7] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                pc = 0x14009f41d;
            }
            0x14009f41d => {
                // 14009f41d  498d40ff  LEA RAX,[R8 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[8]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009f421  4c23c0  AND R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009f424  75aa  JNZ 0x14009f3d0
                pc = if !f.z { 0x14009f3d0 } else { 0x14009f426 };
                continue;
            }
            0x14009f426 => {
                // 14009f426  4c8b1ddb1c3200  MOV R11,qword ptr [0x1403c1108]
                {
                    r[11] = m.read::<64>(0u64.wrapping_add(0x1403c1108u64));
                }
                // 14009f42d  488b1ddc1c3200  MOV RBX,qword ptr [0x1403c1110]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 14009f434  4c8b357d1d3200  MOV R14,qword ptr [0x1403c11b8]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(0x1403c11b8u64));
                }
                pc = 0x14009f43b;
            }
            0x14009f43b => {
                // 14009f43b  4c8bc5  MOV R8,RBP
                {
                    r[8] = r[5];
                }
                // 14009f43e  4d234500  AND R8,qword ptr [R13]
                {
                    let v = r[8] & m.read::<64>(0u64.wrapping_add(r[13]));
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009f442  0f8494000000  JZ 0x14009f4dc
                pc = if f.z { 0x14009f4dc } else { 0x14009f448 };
                continue;
            }
            0x14009f448 => {
                // 14009f448  488b0d191d3200  MOV RCX,qword ptr [0x1403c1168]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(0x1403c1168u64));
                }
                // 14009f44f  488d052a320500  LEA RAX,[0x1400f2680]
                {
                    r[0] = 0u64.wrapping_add(0x1400f2680u64);
                }
                // 14009f456  4c8b6c2478  MOV R13,qword ptr [RSP + 0x78]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x78u64));
                }
                // 14009f45b  4c8d259e0bf6ff  LEA R12,[0x140000000]
                {
                    r[12] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009f462  4889442420  MOV qword ptr [RSP + 0x20],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[0]);
                }
                // 14009f467  660f1f840000000000  NOP word ptr [RAX + RAX*0x1]
                pc = 0x14009f470;
            }
            0x14009f470 => {
                // 14009f470  490fbcd0  BSF RDX,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    if v != 0 {
                        r[2] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f474  4863c2  MOVSXD RAX,EDX
                {
                    r[0] = ((r[2] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 14009f477  89159bfe2500  MOV dword ptr [0x1402ff318],EDX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[2] & 0xffffffffu64);
                }
                // 14009f47d  49858cc480220f00  TEST qword ptr [R12 + RAX*0x8 + 0xf2280],RCX
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf2280u64),
                    ) & r[1];
                    f.logic(v, 64);
                }
                // 14009f485  7522  JNZ 0x14009f4a9
                pc = if !f.z { 0x14009f4a9 } else { 0x14009f487 };
                continue;
            }
            0x14009f487 => {
                // 14009f487  4d859cc480420f00  TEST qword ptr [R12 + RAX*0x8 + 0xf4280],R11
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[12])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4280u64),
                    ) & r[11];
                    f.logic(v, 64);
                }
                // 14009f48f  753a  JNZ 0x14009f4cb
                pc = if !f.z { 0x14009f4cb } else { 0x14009f491 };
                continue;
            }
            0x14009f491 => {
                // 14009f491  41837d7000  CMP dword ptr [R13 + 0x70],0x0
                {
                    f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[13]).wrapping_add(0x70u64)),
                        0x0u64,
                        32,
                        true,
                    );
                }
                // 14009f496  488d0de3330500  LEA RCX,[0x1400f2880]
                {
                    r[1] = 0u64.wrapping_add(0x1400f2880u64);
                }
                // 14009f49d  480f454c2420  CMOVNZ RCX,qword ptr [RSP + 0x20]
                {
                    if !f.z {
                        r[1] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                    }
                }
                // 14009f4a3  488534c1  TEST qword ptr [RCX + RAX*0x8],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[0].wrapping_mul(0x8u64)),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 14009f4a7  741b  JZ 0x14009f4c4
                pc = if f.z { 0x14009f4c4 } else { 0x14009f4a9 };
                continue;
            }
            0x14009f4a9 => {
                // 14009f4a9  418b4d70  MOV ECX,dword ptr [R13 + 0x70]
                {
                    r[1] = (m.read::<32>(0u64.wrapping_add(r[13]).wrapping_add(0x70u64)))
                        & 0xffff_ffff;
                }
                // 14009f4ad  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f4af  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 14009f4b0  83e207  AND EDX,0x7
                {
                    let v = (r[2] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f4b3  03c2  ADD EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f4b5  c1f803  SAR EAX,0x3
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 3u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f4b8  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f4ba  83c106  ADD ECX,0x6
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x6u64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f4bd  443bd1  CMP R10D,ECX
                {
                    f.arithmetic(r[10] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, true);
                }
                // 14009f4c0  440f4cd1  CMOVL R10D,ECX
                {
                    if f.s != f.o {
                        r[10] = (r[1] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                pc = 0x14009f4c4;
            }
            0x14009f4c4 => {
                // 14009f4c4  488b0d9d1c3200  MOV RCX,qword ptr [0x1403c1168]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(0x1403c1168u64));
                }
                pc = 0x14009f4cb;
            }
            0x14009f4cb => {
                // 14009f4cb  498d40ff  LEA RAX,[R8 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[8]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009f4cf  4c23c0  AND R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 14009f4d2  759c  JNZ 0x14009f470
                pc = if !f.z { 0x14009f470 } else { 0x14009f4d4 };
                continue;
            }
            0x14009f4d4 => {
                // 14009f4d4  4c8bac2488000000  MOV R13,qword ptr [RSP + 0x88]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x88u64));
                }
                pc = 0x14009f4dc;
            }
            0x14009f4dc => {
                // 14009f4dc  488b355d1c3200  MOV RSI,qword ptr [0x1403c1140]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1140u64));
                }
                // 14009f4e3  412bfa  SUB EDI,R10D
                {
                    let v = f.arithmetic(r[7] & 0xffffffffu64, r[10] & 0xffffffffu64, 32, true);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 14009f4e6  f3480fb8c5  POPCNT RAX,RBP
                {
                    let v = r[5];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 14009f4eb  f3480fb8cb  POPCNT RCX,RBX
                {
                    let v = r[3];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[1] = u64::from(v.count_ones());
                }
                // 14009f4f0  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f4f2  69c7f4010000  IMUL EAX,EDI,0x1f4
                {
                    r[0] = ((r[7] & 0xffffffffu64).wrapping_mul(0x1f4u64)) & 0xffff_ffff;
                }
                // 14009f4f8  69d15a001400  IMUL EDX,ECX,0x14005a
                {
                    r[2] = ((r[1] & 0xffffffffu64).wrapping_mul(0x14005au64)) & 0xffff_ffff;
                }
                // 14009f4fe  4403c8  ADD R9D,EAX
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 14009f501  4403ca  ADD R9D,EDX
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f504;
            }
            0x14009f504 => {
                // 14009f504  498bc6  MOV RAX,R14
                {
                    r[0] = r[14];
                }
                // 14009f507  4c8d1df20af6ff  LEA R11,[0x140000000]
                {
                    r[11] = 0u64.wrapping_add(0x140000000u64);
                }
                // 14009f50e  25ffff0100  AND EAX,0x1ffff
                {
                    let v = (r[0] & 0xffffffffu64 & 0x1ffffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f513  b900010000  MOV ECX,0x100
                {
                    r[1] = (0x100u64) & 0xffff_ffff;
                }
                // 14009f518  418bd6  MOV EDX,R14D
                {
                    r[2] = (r[14] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f51b  4d8bc6  MOV R8,R14
                {
                    r[8] = r[14];
                }
                // 14009f51e  c1fa07  SAR EDX,0x7
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 7u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f521  81e200fc0300  AND EDX,0x3fc00
                {
                    let v = (r[2] & 0xffffffffu64 & 0x3fc00u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f527  49c1e819  SHR R8,0x19
                {
                    let v = r[8];
                    let n = 25u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 14009f52b  450fbf944310292000  MOVSX R10D,word ptr [R11 + RAX*0x2 + 0x202910]
                {
                    r[10] = (((m.read::<16>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x2u64))
                            .wrapping_add(0x202910u64),
                    )) as i16 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 14009f534  81c20000feff  ADD EDX,0xfffe0000
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, 0xfffe0000u64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f53a  4403157f1c3200  ADD R10D,dword ptr [0x1403c11c0]
                {
                    let v = f.arithmetic(
                        r[10] & 0xffffffffu64,
                        m.read::<32>(0u64.wrapping_add(0x1403c11c0u64)),
                        32,
                        false,
                    );
                    r[10] = (v) & 0xffff_ffff;
                }
                // 14009f541  4181e0ff010000  AND R8D,0x1ff
                {
                    let v = (r[8] & 0xffffffffu64 & 0x1ffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009f548  412bc8  SUB ECX,R8D
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f54b  410fb7c1  MOVZX EAX,R9W
                {
                    r[0] = (r[9] & 0xffffu64) & 0xffff_ffff;
                }
                // 14009f54f  2d00800000  SUB EAX,0x8000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x8000u64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f554  41c1e910  SHR R9D,0x10
                {
                    let v = r[9] & 0xffffffffu64;
                    let n = 16u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[9] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f558  0fafc1  IMUL EAX,ECX
                {
                    r[0] =
                        ((r[0] & 0xffffffffu64).wrapping_mul(r[1] & 0xffffffffu64)) & 0xffff_ffff;
                }
                // 14009f55b  4403d2  ADD R10D,EDX
                {
                    let v = f.arithmetic(r[10] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[10] = (v) & 0xffff_ffff;
                }
                // 14009f55e  4181e900800000  SUB R9D,0x8000
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, 0x8000u64, 32, true);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 14009f565  450fafc8  IMUL R9D,R8D
                {
                    r[9] =
                        ((r[9] & 0xffffffffu64).wrapping_mul(r[8] & 0xffffffffu64)) & 0xffff_ffff;
                }
                // 14009f569  4103c1  ADD EAX,R9D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[9] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f56c  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 14009f56d  0fb6ca  MOVZX ECX,DL
                {
                    r[1] = (r[2] & 0xffu64) & 0xffff_ffff;
                }
                // 14009f570  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f572  b867666666  MOV EAX,0x66666667
                {
                    r[0] = (0x66666667u64) & 0xffff_ffff;
                }
                // 14009f577  c1f908  SAR ECX,0x8
                {
                    let v = r[1] & 0xffffffffu64;
                    let n = 8u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[1] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f57a  4103ca  ADD ECX,R10D
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[10] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f57d  f7e9  IMUL ECX
                {
                    let v = (r[0] as i32 as i128) * ((r[1] & 0xffffffffu64) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 14009f57f  498bce  MOV RCX,R14
                {
                    r[1] = r[14];
                }
                // 14009f582  c1fa02  SAR EDX,0x2
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f585  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f587  48c1e922  SHR RCX,0x22
                {
                    let v = r[1];
                    let n = 34u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 14009f58b  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f58e  81e1ff3f0000  AND ECX,0x3fff
                {
                    let v = (r[1] & 0xffffffffu64 & 0x3fffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f594  03c2  ADD EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f596  420fb68c19c0c32300  MOVZX ECX,byte ptr [RCX + R11*0x1 + 0x23c3c0]
                {
                    r[1] = (m.read::<8>(
                        0u64.wrapping_add(r[1])
                            .wrapping_add(r[11].wrapping_mul(0x1u64))
                            .wrapping_add(0x23c3c0u64),
                    )) & 0xffff_ffff;
                }
                // 14009f59f  0fafc1  IMUL EAX,ECX
                {
                    r[0] =
                        ((r[0] & 0xffffffffu64).wrapping_mul(r[1] & 0xffffffffu64)) & 0xffff_ffff;
                }
                // 14009f5a2  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 14009f5a3  83e27f  AND EDX,0x7f
                {
                    let v = (r[2] & 0xffffffffu64 & 0x7fu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f5a6  448d0402  LEA R8D,[RDX + RAX*0x1]
                {
                    r[8] = (0u64
                        .wrapping_add(r[2])
                        .wrapping_add(r[0].wrapping_mul(0x1u64)))
                        & 0xffff_ffff;
                }
                // 14009f5aa  8b0538b83100  MOV EAX,dword ptr [0x1403bade8]
                {
                    r[0] = (m.read::<32>(0u64.wrapping_add(0x1403bade8u64))) & 0xffff_ffff;
                }
                // 14009f5b0  41c1f807  SAR R8D,0x7
                {
                    let v = r[8] & 0xffffffffu64;
                    let n = 7u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[8] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f5b4  85c0  TEST EAX,EAX
                {
                    let v = (r[0] & 0xffffffffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                }
                // 14009f5b6  742e  JZ 0x14009f5e6
                pc = if f.z { 0x14009f5e6 } else { 0x14009f5b8 };
                continue;
            }
            0x14009f5b8 => {
                // 14009f5b8  8bc8  MOV ECX,EAX
                {
                    r[1] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f5ba  4183c0fe  ADD R8D,-0x2
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, 0xfffffffffffffffeu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009f5be  48330de31b3200  XOR RCX,qword ptr [0x1403c11a8]
                {
                    let v = r[1] ^ m.read::<64>(0u64.wrapping_add(0x1403c11a8u64));
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 14009f5c5  48b8cdcccccccccccccc  MOV RAX,-0x3333333333333333
                {
                    r[0] = 0xcccccccccccccccdu64;
                }
                // 14009f5cf  81e1ff0f0000  AND ECX,0xfff
                {
                    let v = (r[1] & 0xffffffffu64 & 0xfffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 14009f5d5  48f7e1  MUL RCX
                {
                    let v = (r[0] as u128) * ((r[1]) as u128);
                    r[0] = v as u64;
                    r[2] = (v >> 64) as u64;
                }
                // 14009f5d8  48c1ea02  SHR RDX,0x2
                {
                    let v = r[2];
                    let n = 2u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 14009f5dc  488d0492  LEA RAX,[RDX + RDX*0x4]
                {
                    r[0] = 0u64
                        .wrapping_add(r[2])
                        .wrapping_add(r[2].wrapping_mul(0x4u64));
                }
                // 14009f5e0  482bc8  SUB RCX,RAX
                {
                    let v = f.arithmetic(r[1], r[0], 64, true);
                    r[1] = v;
                }
                // 14009f5e3  4403c1  ADD R8D,ECX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f5e6;
            }
            0x14009f5e6 => {
                // 14009f5e6  49bc0000000000500000  MOV R12,0x500000000000
                {
                    r[12] = 0x500000000000u64;
                }
                // 14009f5f0  4d3bfc  CMP R15,R12
                {
                    f.arithmetic(r[15], r[12], 64, true);
                }
                // 14009f5f3  0f85ef010000  JNZ 0x14009f7e8
                pc = if !f.z { 0x14009f7e8 } else { 0x14009f5f9 };
                continue;
            }
            0x14009f5f9 => {
                // 14009f5f9  4c8bcb  MOV R9,RBX
                {
                    r[9] = r[3];
                }
                // 14009f5fc  48b8000000ffffffff00  MOV RAX,0xffffffff000000
                {
                    r[0] = 0xffffffff000000u64;
                }
                // 14009f606  4d234d00  AND R9,qword ptr [R13]
                {
                    let v = r[9] & m.read::<64>(0u64.wrapping_add(r[13]));
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009f60a  bf34000000  MOV EDI,0x34
                {
                    r[7] = (0x34u64) & 0xffff_ffff;
                }
                // 14009f60f  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009f612  0f84df000000  JZ 0x14009f6f7
                pc = if f.z { 0x14009f6f7 } else { 0x14009f618 };
                continue;
            }
            0x14009f618 => {
                // 14009f618  4c8b35291b3200  MOV R14,qword ptr [0x1403c1148]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                }
                // 14009f61f  49bc0000000000ff0000  MOV R12,0xff0000000000
                {
                    r[12] = 0xff0000000000u64;
                }
                // 14009f629  48bd000000000000ff00  MOV RBP,0xff000000000000
                {
                    r[5] = 0xff000000000000u64;
                }
                // 14009f633  49bd00000000ff000000  MOV R13,0xff00000000
                {
                    r[13] = 0xff00000000u64;
                }
                // 14009f63d  0f1f00  NOP dword ptr [RAX]
                pc = 0x14009f640;
            }
            0x14009f640 => {
                // 14009f640  490fbcc1  BSF RAX,R9
                {
                    let v = r[9];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f644  8905cefc2500  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009f64a  4898  CDQE
                {
                    r[0] = r[0] as i32 as i64 as u64;
                }
                // 14009f64c  4985b4c380440f00  TEST qword ptr [R11 + RAX*0x8 + 0xf4480],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4480u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 14009f654  7474  JZ 0x14009f6ca
                pc = if f.z { 0x14009f6ca } else { 0x14009f656 };
                continue;
            }
            0x14009f656 => {
                // 14009f656  4d85b4c380420f00  TEST qword ptr [R11 + RAX*0x8 + 0xf4280],R14
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4280u64),
                    ) & r[14];
                    f.logic(v, 64);
                }
                // 14009f65e  746a  JZ 0x14009f6ca
                pc = if f.z { 0x14009f6ca } else { 0x14009f660 };
                continue;
            }
            0x14009f660 => {
                // 14009f660  49859cc380460f00  TEST qword ptr [R11 + RAX*0x8 + 0xf4680],RBX
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4680u64),
                    ) & r[3];
                    f.logic(v, 64);
                }
                // 14009f668  7560  JNZ 0x14009f6ca
                pc = if !f.z { 0x14009f6ca } else { 0x14009f66a };
                continue;
            }
            0x14009f66a => {
                // 14009f66a  498b94c3f0de0e00  MOV RDX,qword ptr [R11 + RAX*0x8 + 0xedef0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xedef0u64),
                    );
                }
                // 14009f672  482315ff1a3200  AND RDX,qword ptr [0x1403c1178]
                {
                    let v = r[2] & m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 14009f679  490faf94c3f0fe0e00  IMUL RDX,qword ptr [R11 + RAX*0x8 + 0xefef0]
                {
                    r[2] = (r[2]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[11])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xefef0u64),
                        ),
                    );
                }
                // 14009f682  418b8c8300080f00  MOV ECX,dword ptr [R11 + RAX*0x4 + 0xf0800]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0800u64),
                    )) & 0xffff_ffff;
                }
                // 14009f68a  498b84c3f0020f00  MOV RAX,qword ptr [R11 + RAX*0x8 + 0xf02f0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf02f0u64),
                    );
                }
                // 14009f692  48d3ea  SHR RDX,CL
                {
                    let v = r[2];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 14009f695  4c8534d0  TEST qword ptr [RAX + RDX*0x8],R14
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[2].wrapping_mul(0x8u64)),
                    ) & r[14];
                    f.logic(v, 64);
                }
                // 14009f699  742f  JZ 0x14009f6ca
                pc = if f.z { 0x14009f6ca } else { 0x14009f69b };
                continue;
            }
            0x14009f69b => {
                // 14009f69b  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 14009f69e  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 14009f6a1  4923c1  AND RAX,R9
                {
                    let v = r[0] & r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f6a4  4885c5  TEST RBP,RAX
                {
                    let v = r[5] & r[0];
                    f.logic(v, 64);
                }
                // 14009f6a7  7407  JZ 0x14009f6b0
                pc = if f.z { 0x14009f6b0 } else { 0x14009f6a9 };
                continue;
            }
            0x14009f6a9 => {
                // 14009f6a9  b8a0000000  MOV EAX,0xa0
                {
                    r[0] = (0xa0u64) & 0xffff_ffff;
                }
                // 14009f6ae  eb17  JMP 0x14009f6c7
                pc = 0x14009f6c7;
                continue;
            }
            0x14009f6b0 => {
                // 14009f6b0  4985c4  TEST R12,RAX
                {
                    let v = r[12] & r[0];
                    f.logic(v, 64);
                }
                // 14009f6b3  7407  JZ 0x14009f6bc
                pc = if f.z { 0x14009f6bc } else { 0x14009f6b5 };
                continue;
            }
            0x14009f6b5 => {
                // 14009f6b5  b864000000  MOV EAX,0x64
                {
                    r[0] = (0x64u64) & 0xffff_ffff;
                }
                // 14009f6ba  eb0b  JMP 0x14009f6c7
                pc = 0x14009f6c7;
                continue;
            }
            0x14009f6bc => {
                // 14009f6bc  4985c5  TEST R13,RAX
                {
                    let v = r[13] & r[0];
                    f.logic(v, 64);
                }
                // 14009f6bf  b816000000  MOV EAX,0x16
                {
                    r[0] = (0x16u64) & 0xffff_ffff;
                }
                // 14009f6c4  0f45c7  CMOVNZ EAX,EDI
                {
                    if !f.z {
                        r[0] = (r[7] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                pc = 0x14009f6c7;
            }
            0x14009f6c7 => {
                // 14009f6c7  442bc0  SUB R8D,EAX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[8] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f6ca;
            }
            0x14009f6ca => {
                // 14009f6ca  498d41ff  LEA RAX,[R9 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[9]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009f6ce  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009f6d1  0f8569ffffff  JNZ 0x14009f640
                pc = if !f.z { 0x14009f640 } else { 0x14009f6d7 };
                continue;
            }
            0x14009f6d7 => {
                // 14009f6d7  4c8bac2488000000  MOV R13,qword ptr [RSP + 0x88]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x88u64));
                }
                // 14009f6df  49bc0000000000500000  MOV R12,0x500000000000
                {
                    r[12] = 0x500000000000u64;
                }
                // 14009f6e9  488b2d281a3200  MOV RBP,qword ptr [0x1403c1118]
                {
                    r[5] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 14009f6f0  4c8b35c11a3200  MOV R14,qword ptr [0x1403c11b8]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(0x1403c11b8u64));
                }
                pc = 0x14009f6f7;
            }
            0x14009f6f7 => {
                // 14009f6f7  4c8bcd  MOV R9,RBP
                {
                    r[9] = r[5];
                }
                // 14009f6fa  48b800ffffffff000000  MOV RAX,0xffffffff00
                {
                    r[0] = 0xffffffff00u64;
                }
                // 14009f704  4d234d00  AND R9,qword ptr [R13]
                {
                    let v = r[9] & m.read::<64>(0u64.wrapping_add(r[13]));
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009f708  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009f70b  0f84b7000000  JZ 0x14009f7c8
                pc = if f.z { 0x14009f7c8 } else { 0x14009f711 };
                continue;
            }
            0x14009f711 => {
                // 14009f711  488b0d301a3200  MOV RCX,qword ptr [0x1403c1148]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                }
                // 14009f718  bb000000ff  MOV EBX,0xff000000
                {
                    r[3] = (0xff000000u64) & 0xffff_ffff;
                }
                // 14009f71d  0f1f00  NOP dword ptr [RAX]
                pc = 0x14009f720;
            }
            0x14009f720 => {
                // 14009f720  490fbcc1  BSF RAX,R9
                {
                    let v = r[9];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 14009f724  8905eefb2500  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 14009f72a  4898  CDQE
                {
                    r[0] = r[0] as i32 as i64 as u64;
                }
                // 14009f72c  49858cc380420f00  TEST qword ptr [R11 + RAX*0x8 + 0xf4280],RCX
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4280u64),
                    ) & r[1];
                    f.logic(v, 64);
                }
                // 14009f734  0f8481000000  JZ 0x14009f7bb
                pc = if f.z { 0x14009f7bb } else { 0x14009f73a };
                continue;
            }
            0x14009f73a => {
                // 14009f73a  4985b4c380440f00  TEST qword ptr [R11 + RAX*0x8 + 0xf4480],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4480u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 14009f742  7477  JZ 0x14009f7bb
                pc = if f.z { 0x14009f7bb } else { 0x14009f744 };
                continue;
            }
            0x14009f744 => {
                // 14009f744  4985acc380480f00  TEST qword ptr [R11 + RAX*0x8 + 0xf4880],RBP
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4880u64),
                    ) & r[5];
                    f.logic(v, 64);
                }
                // 14009f74c  756d  JNZ 0x14009f7bb
                pc = if !f.z { 0x14009f7bb } else { 0x14009f74e };
                continue;
            }
            0x14009f74e => {
                // 14009f74e  498b94c3f0de0e00  MOV RDX,qword ptr [R11 + RAX*0x8 + 0xedef0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xedef0u64),
                    );
                }
                // 14009f756  4823151b1a3200  AND RDX,qword ptr [0x1403c1178]
                {
                    let v = r[2] & m.read::<64>(0u64.wrapping_add(0x1403c1178u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 14009f75d  490faf94c3f0fe0e00  IMUL RDX,qword ptr [R11 + RAX*0x8 + 0xefef0]
                {
                    r[2] = (r[2]).wrapping_mul(
                        m.read::<64>(
                            0u64.wrapping_add(r[11])
                                .wrapping_add(r[0].wrapping_mul(0x8u64))
                                .wrapping_add(0xefef0u64),
                        ),
                    );
                }
                // 14009f766  418b8c8300080f00  MOV ECX,dword ptr [R11 + RAX*0x4 + 0xf0800]
                {
                    r[1] = (m.read::<32>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0xf0800u64),
                    )) & 0xffff_ffff;
                }
                // 14009f76e  498b84c3f0020f00  MOV RAX,qword ptr [R11 + RAX*0x8 + 0xf02f0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[11])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf02f0u64),
                    );
                }
                // 14009f776  48d3ea  SHR RDX,CL
                {
                    let v = r[2];
                    let n = (r[1] & 0xffu64 & 63) as u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 14009f779  488534d0  TEST qword ptr [RAX + RDX*0x8],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[2].wrapping_mul(0x8u64)),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 14009f77d  7435  JZ 0x14009f7b4
                pc = if f.z { 0x14009f7b4 } else { 0x14009f77f };
                continue;
            }
            0x14009f77f => {
                // 14009f77f  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 14009f782  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 14009f785  4923c1  AND RAX,R9
                {
                    let v = r[0] & r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f788  48a900ff0000  TEST RAX,0xff00
                {
                    let v = r[0] & 0xff00u64;
                    f.logic(v, 64);
                }
                // 14009f78e  7407  JZ 0x14009f797
                pc = if f.z { 0x14009f797 } else { 0x14009f790 };
                continue;
            }
            0x14009f790 => {
                // 14009f790  b8a0000000  MOV EAX,0xa0
                {
                    r[0] = (0xa0u64) & 0xffff_ffff;
                }
                // 14009f795  eb1a  JMP 0x14009f7b1
                pc = 0x14009f7b1;
                continue;
            }
            0x14009f797 => {
                // 14009f797  48a90000ff00  TEST RAX,0xff0000
                {
                    let v = r[0] & 0xff0000u64;
                    f.logic(v, 64);
                }
                // 14009f79d  7407  JZ 0x14009f7a6
                pc = if f.z { 0x14009f7a6 } else { 0x14009f79f };
                continue;
            }
            0x14009f79f => {
                // 14009f79f  b864000000  MOV EAX,0x64
                {
                    r[0] = (0x64u64) & 0xffff_ffff;
                }
                // 14009f7a4  eb0b  JMP 0x14009f7b1
                pc = 0x14009f7b1;
                continue;
            }
            0x14009f7a6 => {
                // 14009f7a6  4885c3  TEST RBX,RAX
                {
                    let v = r[3] & r[0];
                    f.logic(v, 64);
                }
                // 14009f7a9  b816000000  MOV EAX,0x16
                {
                    r[0] = (0x16u64) & 0xffff_ffff;
                }
                // 14009f7ae  0f45c7  CMOVNZ EAX,EDI
                {
                    if !f.z {
                        r[0] = (r[7] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                pc = 0x14009f7b1;
            }
            0x14009f7b1 => {
                // 14009f7b1  4403c0  ADD R8D,EAX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f7b4;
            }
            0x14009f7b4 => {
                // 14009f7b4  488b0d8d193200  MOV RCX,qword ptr [0x1403c1148]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(0x1403c1148u64));
                }
                pc = 0x14009f7bb;
            }
            0x14009f7bb => {
                // 14009f7bb  498d41ff  LEA RAX,[R9 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[9]).wrapping_add(0xffffffffffffffffu64);
                }
                // 14009f7bf  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 14009f7c2  0f8558ffffff  JNZ 0x14009f720
                pc = if !f.z { 0x14009f720 } else { 0x14009f7c8 };
                continue;
            }
            0x14009f7c8 => {
                // 14009f7c8  440faf054cf32500  IMUL R8D,dword ptr [0x1402feb1c]
                {
                    r[8] = ((r[8] & 0xffffffffu64)
                        .wrapping_mul(m.read::<32>(0u64.wrapping_add(0x1402feb1cu64))))
                        & 0xffff_ffff;
                }
                // 14009f7d0  b81f85eb51  MOV EAX,0x51eb851f
                {
                    r[0] = (0x51eb851fu64) & 0xffff_ffff;
                }
                // 14009f7d5  41f7e8  IMUL R8D
                {
                    let v = (r[0] as i32 as i128) * ((r[8] & 0xffffffffu64) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 14009f7d8  448bc2  MOV R8D,EDX
                {
                    r[8] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f7db  41c1f805  SAR R8D,0x5
                {
                    let v = r[8] & 0xffffffffu64;
                    let n = 5u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[8] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f7df  418bc0  MOV EAX,R8D
                {
                    r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f7e2  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f7e5  4403c0  ADD R8D,EAX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f7e8;
            }
            0x14009f7e8 => {
                // 14009f7e8  448b0d25f32500  MOV R9D,dword ptr [0x1402feb14]
                {
                    r[9] = (m.read::<32>(0u64.wrapping_add(0x1402feb14u64))) & 0xffff_ffff;
                }
                // 14009f7ef  4d3bfc  CMP R15,R12
                {
                    f.arithmetic(r[15], r[12], 64, true);
                }
                // 14009f7f2  751c  JNZ 0x14009f810
                pc = if !f.z { 0x14009f810 } else { 0x14009f7f4 };
                continue;
            }
            0x14009f7f4 => {
                // 14009f7f4  b899999999  MOV EAX,0x99999999
                {
                    r[0] = (0x99999999u64) & 0xffff_ffff;
                }
                // 14009f7f9  f72d3db63100  IMUL dword ptr [0x1403bae3c]
                {
                    let v = (r[0] as i32 as i128)
                        * ((m.read::<32>(0u64.wrapping_add(0x1403bae3cu64))) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 14009f7ff  c1fa02  SAR EDX,0x2
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f802  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f804  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f807  03d0  ADD EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f809  410fafd1  IMUL EDX,R9D
                {
                    r[2] =
                        ((r[2] & 0xffffffffu64).wrapping_mul(r[9] & 0xffffffffu64)) & 0xffff_ffff;
                }
                // 14009f80d  4403c2  ADD R8D,EDX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f810;
            }
            0x14009f810 => {
                // 14009f810  48b9000000003c000000  MOV RCX,0x3c00000000
                {
                    r[1] = 0x3c00000000u64;
                }
                // 14009f81a  498bc6  MOV RAX,R14
                {
                    r[0] = r[14];
                }
                // 14009f81d  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 14009f820  48b90000000024000000  MOV RCX,0x2400000000
                {
                    r[1] = 0x2400000000u64;
                }
                // 14009f82a  483bc1  CMP RAX,RCX
                {
                    f.arithmetic(r[0], r[1], 64, true);
                }
                // 14009f82d  740f  JZ 0x14009f83e
                pc = if f.z { 0x14009f83e } else { 0x14009f82f };
                continue;
            }
            0x14009f82f => {
                // 14009f82f  48b90000000018000000  MOV RCX,0x1800000000
                {
                    r[1] = 0x1800000000u64;
                }
                // 14009f839  483bc1  CMP RAX,RCX
                {
                    f.arithmetic(r[0], r[1], 64, true);
                }
                // 14009f83c  7547  JNZ 0x14009f885
                pc = if !f.z { 0x14009f885 } else { 0x14009f83e };
                continue;
            }
            0x14009f83e => {
                // 14009f83e  b899999999  MOV EAX,0x99999999
                {
                    r[0] = (0x99999999u64) & 0xffff_ffff;
                }
                // 14009f843  f72defb53100  IMUL dword ptr [0x1403bae38]
                {
                    let v = (r[0] as i32 as i128)
                        * ((m.read::<32>(0u64.wrapping_add(0x1403bae38u64))) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 14009f849  c1fa02  SAR EDX,0x2
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f84c  8bca  MOV ECX,EDX
                {
                    r[1] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f84e  c1e91f  SHR ECX,0x1f
                {
                    let v = r[1] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[1] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f851  03d1  ADD EDX,ECX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f853  48b900000000c0ff0000  MOV RCX,0xffc000000000
                {
                    r[1] = 0xffc000000000u64;
                }
                // 14009f85d  410fafd1  IMUL EDX,R9D
                {
                    r[2] =
                        ((r[2] & 0xffffffffu64).wrapping_mul(r[9] & 0xffffffffu64)) & 0xffff_ffff;
                }
                // 14009f861  4403c2  ADD R8D,EDX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 14009f864  4c85f1  TEST RCX,R14
                {
                    let v = r[1] & r[14];
                    f.logic(v, 64);
                }
                // 14009f867  751c  JNZ 0x14009f885
                pc = if !f.z { 0x14009f885 } else { 0x14009f869 };
                continue;
            }
            0x14009f869 => {
                // 14009f869  b899999999  MOV EAX,0x99999999
                {
                    r[0] = (0x99999999u64) & 0xffff_ffff;
                }
                // 14009f86e  f72d88b53100  IMUL dword ptr [0x1403badfc]
                {
                    let v = (r[0] as i32 as i128)
                        * ((m.read::<32>(0u64.wrapping_add(0x1403badfcu64))) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 14009f874  c1fa02  SAR EDX,0x2
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f877  8bca  MOV ECX,EDX
                {
                    r[1] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f879  c1e91f  SHR ECX,0x1f
                {
                    let v = r[1] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[1] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f87c  03d1  ADD EDX,ECX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[1] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f87e  410fafd1  IMUL EDX,R9D
                {
                    r[2] =
                        ((r[2] & 0xffffffffu64).wrapping_mul(r[9] & 0xffffffffu64)) & 0xffff_ffff;
                }
                // 14009f882  4403c2  ADD R8D,EDX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[8] = (v) & 0xffff_ffff;
                }
                pc = 0x14009f885;
            }
            0x14009f885 => {
                // 14009f885  488b4c2478  MOV RCX,qword ptr [RSP + 0x78]
                {
                    r[1] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x78u64));
                }
                // 14009f88a  4c8d9c24a0000000  LEA R11,[RSP + 0xa0]
                {
                    r[11] = 0u64.wrapping_add(r[4]).wrapping_add(0xa0u64);
                }
                // 14009f892  498b5b38  MOV RBX,qword ptr [R11 + 0x38]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64));
                }
                // 14009f896  b867666666  MOV EAX,0x66666667
                {
                    r[0] = (0x66666667u64) & 0xffff_ffff;
                }
                // 14009f89b  498b6b40  MOV RBP,qword ptr [R11 + 0x40]
                {
                    r[5] = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x40u64));
                }
                // 14009f89f  498b7348  MOV RSI,qword ptr [R11 + 0x48]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x48u64));
                }
                // 14009f8a3  41f7ea  IMUL R10D
                {
                    let v = (r[0] as i32 as i128) * ((r[10] & 0xffffffffu64) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 14009f8a6  c6417c00  MOV byte ptr [RCX + 0x7c],0x0
                {
                    m.write::<8>(0u64.wrapping_add(r[1]).wrapping_add(0x7cu64), 0x0u64);
                }
                // 14009f8aa  c1fa02  SAR EDX,0x2
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f8ad  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f8af  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 14009f8b2  03d0  ADD EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 14009f8b4  418bc0  MOV EAX,R8D
                {
                    r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f8b7  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 14009f8b9  89416c  MOV dword ptr [RCX + 0x6c],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[1]).wrapping_add(0x6cu64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 14009f8bc  418bc0  MOV EAX,R8D
                {
                    r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 14009f8bf  498be3  MOV RSP,R11
                {
                    r[4] = r[11];
                }
                // 14009f8c2  415f  POP R15
                {
                    r[15] = m.read::<64>(r[4]);
                    r[4] = r[4].wrapping_add(8);
                }
                // 14009f8c4  415e  POP R14
                {
                    r[14] = m.read::<64>(r[4]);
                    r[4] = r[4].wrapping_add(8);
                }
                // 14009f8c6  415d  POP R13
                {
                    r[13] = m.read::<64>(r[4]);
                    r[4] = r[4].wrapping_add(8);
                }
                // 14009f8c8  415c  POP R12
                {
                    r[12] = m.read::<64>(r[4]);
                    r[4] = r[4].wrapping_add(8);
                }
                // 14009f8ca  5f  POP RDI
                {
                    r[7] = m.read::<64>(r[4]);
                    r[4] = r[4].wrapping_add(8);
                }
                // 14009f8cb  c3  RET
                *registers = r;
                *simd = x;
                return r[0];
            }
            _ => unreachable!("unregistered original basic block {pc:x}"),
        }
    }
}

// Source SHA-256: 264f2eede4c737f938c726e3c91e5ce07460e998110cf197abfc463dfa4622ae
pub(super) fn pawn_1400ca0f0(
    m: &mut Memory,
    registers: &mut [u64; 16],
    simd: &mut [u128; 2],
) -> u64 {
    let mut r = *registers;
    let mut x = *simd;
    let mut f = Flags::default();
    let mut pc = 0x1400ca0f0u64;
    loop {
        match pc {
            0x1400ca0f0 => {
                // 1400ca0f0  4055  PUSH RBP
                {
                    r[4] = r[4].wrapping_sub(8);
                    m.write::<64>(r[4], r[5]);
                }
                // 1400ca0f2  488d6c2490  LEA RBP,[RSP + -0x70]
                {
                    r[5] = 0u64.wrapping_add(r[4]).wrapping_add(0xffffffffffffff90u64);
                }
                // 1400ca0f7  4881ec70010000  SUB RSP,0x170
                {
                    let v = f.arithmetic(r[4], 0x170u64, 64, true);
                    r[4] = v;
                }
                // 1400ca0fe  488b05ab702f00  MOV RAX,qword ptr [0x1403c11b0]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(0x1403c11b0u64));
                }
                // 1400ca105  4c8bd8  MOV R11,RAX
                {
                    r[11] = r[0];
                }
                // 1400ca108  4889442428  MOV qword ptr [RSP + 0x28],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64), r[0]);
                }
                // 1400ca10d  49c1eb07  SHR R11,0x7
                {
                    let v = r[11];
                    let n = 7u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[11] = v;
                    }
                }
                // 1400ca111  4181e3fe3f0000  AND R11D,0x3ffe
                {
                    let v = (r[11] & 0xffffffffu64 & 0x3ffeu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[11] = (v) & 0xffff_ffff;
                }
                // 1400ca118  49c1e306  SHL R11,0x6
                {
                    let v = r[11];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[11] = v;
                    }
                }
                // 1400ca11c  4c031dfdb32800  ADD R11,qword ptr [0x140355520]
                {
                    let v = f.arithmetic(
                        r[11],
                        m.read::<64>(0u64.wrapping_add(0x140355520u64)),
                        64,
                        false,
                    );
                    r[11] = v;
                }
                // 1400ca123  49394318  CMP qword ptr [R11 + 0x18],RAX
                {
                    f.arithmetic(
                        m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x18u64)),
                        r[0],
                        64,
                        true,
                    );
                }
                // 1400ca127  7510  JNZ 0x1400ca139
                pc = if !f.z { 0x1400ca139 } else { 0x1400ca129 };
                continue;
            }
            0x1400ca129 => {
                // 1400ca129  41ff433c  INC dword ptr [R11 + 0x3c]
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x3cu64)),
                        1u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x3cu64), v);
                }
                // 1400ca12d  498bc3  MOV RAX,R11
                {
                    r[0] = r[11];
                }
                // 1400ca130  4881c470010000  ADD RSP,0x170
                {
                    let v = f.arithmetic(r[4], 0x170u64, 64, false);
                    r[4] = v;
                }
                // 1400ca137  5d  POP RBP
                {
                    r[5] = m.read::<64>(r[4]);
                    r[4] = r[4].wrapping_add(8);
                }
                // 1400ca138  c3  RET
                *registers = r;
                *simd = x;
                return r[0];
            }
            0x1400ca139 => {
                // 1400ca139  4c8b0d30702f00  MOV R9,qword ptr [0x1403c1170]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(0x1403c1170u64));
                }
                // 1400ca140  4c8b15d16f2f00  MOV R10,qword ptr [0x1403c1118]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400ca147  48899c2468010000  MOV qword ptr [RSP + 0x168],RBX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x168u64), r[3]);
                }
                // 1400ca14f  488b1dba6f2f00  MOV RBX,qword ptr [0x1403c1110]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400ca156  4889b42460010000  MOV qword ptr [RSP + 0x160],RSI
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x160u64), r[6]);
                }
                // 1400ca15e  4889bc2458010000  MOV qword ptr [RSP + 0x158],RDI
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x158u64), r[7]);
                }
                // 1400ca166  48633d07702f00  MOVSXD RDI,dword ptr [0x1403c1174]
                {
                    r[7] = ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 1400ca16d  4c89a42450010000  MOV qword ptr [RSP + 0x150],R12
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x150u64), r[12]);
                }
                // 1400ca175  8bc7  MOV EAX,EDI
                {
                    r[0] = (r[7] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca177  4c89ac2448010000  MOV qword ptr [RSP + 0x148],R13
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x148u64), r[13]);
                }
                // 1400ca17f  4533e4  XOR R12D,R12D
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[12] = (v) & 0xffff_ffff;
                }
                // 1400ca182  4133c1  XOR EAX,R9D
                {
                    let v = (r[0] & 0xffffffffu64 ^ r[9] & 0xffffffffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca185  4c89b42440010000  MOV qword ptr [RSP + 0x140],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x140u64), r[14]);
                }
                // 1400ca18d  4c89bc2438010000  MOV qword ptr [RSP + 0x138],R15
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x138u64), r[15]);
                }
                // 1400ca195  458bc4  MOV R8D,R12D
                {
                    r[8] = (r[12] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca198  4c8d3d615ef3ff  LEA R15,[0x140000000]
                {
                    r[15] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca19f  a804  TEST AL,0x4
                {
                    let v = (r[0] & 0xffu64 & 0x4u64) & 0xffu64;
                    f.logic(v, 8);
                }
                // 1400ca1a1  7433  JZ 0x1400ca1d6
                pc = if f.z { 0x1400ca1d6 } else { 0x1400ca1a3 };
                continue;
            }
            0x1400ca1a3 => {
                // 1400ca1a3  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 1400ca1a6  4c8bc3  MOV R8,RBX
                {
                    r[8] = r[3];
                }
                // 1400ca1a9  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca1ac  4d0bc2  OR R8,R10
                {
                    let v = r[8] | r[10];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400ca1af  498b94c7102b1100  MOV RDX,qword ptr [R15 + RAX*0x8 + 0x112b10]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0x112b10u64),
                    );
                }
                // 1400ca1b7  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400ca1ba  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400ca1bd  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca1c0  4c23c2  AND R8,RDX
                {
                    let v = r[8] & r[2];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400ca1c3  f3480fb8c8  POPCNT RCX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[1] = u64::from(v.count_ones());
                }
                // 1400ca1c8  f3490fb8c0  POPCNT RAX,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 1400ca1cd  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca1cf  4469c10a001e00  IMUL R8D,ECX,0x1e000a
                {
                    r[8] = ((r[1] & 0xffffffffu64).wrapping_mul(0x1e000au64)) & 0xffff_ffff;
                }
                pc = 0x1400ca1d6;
            }
            0x1400ca1d6 => {
                // 1400ca1d6  49befefefefefefefe00  MOV R14,0xfefefefefefefe
                {
                    r[14] = 0xfefefefefefefeu64;
                }
                // 1400ca1e0  4d896320  MOV qword ptr [R11 + 0x20],R12
                {
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x20u64), r[12]);
                }
                // 1400ca1e4  48b97f7f7f7f7f7fffff  MOV RCX,-0x808080808081
                {
                    r[1] = 0xffff7f7f7f7f7f7fu64;
                }
                // 1400ca1ee  4d896328  MOV qword ptr [R11 + 0x28],R12
                {
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x28u64), r[12]);
                }
                // 1400ca1f2  498bf2  MOV RSI,R10
                {
                    r[6] = r[10];
                }
                // 1400ca1f5  4d8923  MOV qword ptr [R11],R12
                {
                    m.write::<64>(0u64.wrapping_add(r[11]), r[12]);
                }
                // 1400ca1f8  4c8b2d116f2f00  MOV R13,qword ptr [0x1403c1110]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400ca1ff  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400ca202  4823c1  AND RAX,RCX
                {
                    let v = r[0] & r[1];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca205  48c1ee09  SHR RSI,0x9
                {
                    let v = r[6];
                    let n = 9u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[6] = v;
                    }
                }
                // 1400ca209  48c1e009  SHL RAX,0x9
                {
                    let v = r[0];
                    let n = 9u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400ca20d  48b97f7f7f7f7f7f7f00  MOV RCX,0x7f7f7f7f7f7f7f
                {
                    r[1] = 0x7f7f7f7f7f7f7fu64;
                }
                // 1400ca217  4823f1  AND RSI,RCX
                {
                    let v = r[6] & r[1];
                    f.logic(v, 64);
                    r[6] = v;
                }
                // 1400ca21a  48898598000000  MOV qword ptr [RBP + 0x98],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x98u64), r[0]);
                }
                // 1400ca221  498bca  MOV RCX,R10
                {
                    r[1] = r[10];
                }
                // 1400ca224  4889742418  MOV qword ptr [RSP + 0x18],RSI
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x18u64), r[6]);
                }
                // 1400ca229  48c1e907  SHR RCX,0x7
                {
                    let v = r[1];
                    let n = 7u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[1] = v;
                    }
                }
                // 1400ca22d  488bd3  MOV RDX,RBX
                {
                    r[2] = r[3];
                }
                // 1400ca230  4923ce  AND RCX,R14
                {
                    let v = r[1] & r[14];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 1400ca233  4c89a590000000  MOV qword ptr [RBP + 0x90],R12
                {
                    m.write::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64), r[12]);
                }
                // 1400ca23a  4923d6  AND RDX,R14
                {
                    let v = r[2] & r[14];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400ca23d  48894c2410  MOV qword ptr [RSP + 0x10],RCX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x10u64), r[1]);
                }
                // 1400ca242  48c1e207  SHL RDX,0x7
                {
                    let v = r[2];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400ca246  4c8bf0  MOV R14,RAX
                {
                    r[14] = r[0];
                }
                // 1400ca249  488bc1  MOV RAX,RCX
                {
                    r[0] = r[1];
                }
                // 1400ca24c  48891424  MOV qword ptr [RSP],RDX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]), r[2]);
                }
                // 1400ca250  4c0bf2  OR R14,RDX
                {
                    let v = r[14] | r[2];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 1400ca253  4963c9  MOVSXD RCX,R9D
                {
                    r[1] = ((r[9] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca256  480bc6  OR RAX,RSI
                {
                    let v = r[0] | r[6];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca259  4c89742408  MOV qword ptr [RSP + 0x8],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x8u64), r[14]);
                }
                // 1400ca25e  4889442420  MOV qword ptr [RSP + 0x20],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64), r[0]);
                }
                // 1400ca263  418d503c  LEA EDX,[R8 + 0x3c]
                {
                    r[2] = (0u64.wrapping_add(r[8]).wrapping_add(0x3cu64)) & 0xffff_ffff;
                }
                // 1400ca267  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400ca26a  498b8ccff0f40e00  MOV RCX,qword ptr [R15 + RCX*0x8 + 0xef4f0]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[1].wrapping_mul(0x8u64))
                            .wrapping_add(0xef4f0u64),
                    );
                }
                // 1400ca272  4823c8  AND RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 1400ca275  498bc6  MOV RAX,R14
                {
                    r[0] = r[14];
                }
                // 1400ca278  4985ca  TEST R10,RCX
                {
                    let v = r[10] & r[1];
                    f.logic(v, 64);
                }
                // 1400ca27b  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400ca27e  41ba3f000000  MOV R10D,0x3f
                {
                    r[10] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400ca284  4d8bf4  MOV R14,R12
                {
                    r[14] = r[12];
                }
                // 1400ca287  410f44d0  CMOVZ EDX,R8D
                {
                    if f.z {
                        r[2] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 1400ca28b  44899588000000  MOV dword ptr [RBP + 0x88],R10D
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x88u64),
                        r[10] & 0xffffffffu64,
                    );
                }
                // 1400ca292  4c8d05675df3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca299  44899580000000  MOV dword ptr [RBP + 0x80],R10D
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x80u64),
                        r[10] & 0xffffffffu64,
                    );
                }
                // 1400ca2a0  498b8cf8f0f40e00  MOV RCX,qword ptr [R8 + RDI*0x8 + 0xef4f0]
                {
                    r[1] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0xef4f0u64),
                    );
                }
                // 1400ca2a8  458bfa  MOV R15D,R10D
                {
                    r[15] = (r[10] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca2ab  4823c8  AND RCX,RAX
                {
                    let v = r[1] & r[0];
                    f.logic(v, 64);
                    r[1] = v;
                }
                // 1400ca2ae  4885cb  TEST RBX,RCX
                {
                    let v = r[3] & r[1];
                    f.logic(v, 64);
                }
                // 1400ca2b1  8d4ac4  LEA ECX,[RDX + -0x3c]
                {
                    r[1] =
                        (0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffc4u64)) & 0xffff_ffff;
                }
                // 1400ca2b4  0f44ca  CMOVZ ECX,EDX
                {
                    if f.z {
                        r[1] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 1400ca2b7  4d85ed  TEST R13,R13
                {
                    let v = r[13];
                    f.logic(v, 64);
                }
                // 1400ca2ba  0f840d050000  JZ 0x1400ca7cd
                pc = if f.z { 0x1400ca7cd } else { 0x1400ca2c0 };
                continue;
            }
            0x1400ca2c0 => {
                // 1400ca2c0  488b3d516e2f00  MOV RDI,qword ptr [0x1403c1118]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400ca2c7  660f1f840000000000  NOP word ptr [RAX + RAX*0x1]
                pc = 0x1400ca2d0;
            }
            0x1400ca2d0 => {
                // 1400ca2d0  490fbcf5  BSF RSI,R13
                {
                    let v = r[13];
                    f.z = v == 0;
                    if v != 0 {
                        r[6] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca2d4  4c63ce  MOVSXD R9,ESI
                {
                    r[9] = ((r[6] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca2d7  89353b502300  MOV dword ptr [0x1402ff318],ESI
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[6] & 0xffffffffu64);
                }
                // 1400ca2dd  4a8d1ccd00000000  LEA RBX,[R9*0x8]
                {
                    r[3] = 0u64.wrapping_add(r[9].wrapping_mul(0x8u64));
                }
                // 1400ca2e5  83fe30  CMP ESI,0x30
                {
                    f.arithmetic(r[6] & 0xffffffffu64, 0x30u64, 32, true);
                }
                // 1400ca2e8  7c11  JL 0x1400ca2fb
                pc = if f.s != f.o { 0x1400ca2fb } else { 0x1400ca2ea };
                continue;
            }
            0x1400ca2ea => {
                // 1400ca2ea  4a8b840380480f00  MOV RAX,qword ptr [RBX + R8*0x1 + 0xf4880]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[8].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4880u64),
                    );
                }
                // 1400ca2f2  49094320  OR qword ptr [R11 + 0x20],RAX
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x20u64)) | r[0];
                    f.logic(v, 64);
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x20u64), v);
                }
                // 1400ca2f6  e99b000000  JMP 0x1400ca396
                pc = 0x1400ca396;
                continue;
            }
            0x1400ca2fb => {
                // 1400ca2fb  8d4608  LEA EAX,[RSI + 0x8]
                {
                    r[0] = (0u64.wrapping_add(r[6]).wrapping_add(0x8u64)) & 0xffff_ffff;
                }
                // 1400ca2fe  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca301  4d8b94d080360f00  MOV R10,qword ptr [R8 + RDX*0x8 + 0xf3680]
                {
                    r[10] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3680u64),
                    );
                }
                // 1400ca309  4a8b940380440f00  MOV RDX,qword ptr [RBX + R8*0x1 + 0xf4480]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[8].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4480u64),
                    );
                }
                // 1400ca311  4823d7  AND RDX,RDI
                {
                    let v = r[2] & r[7];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400ca314  742e  JZ 0x1400ca344
                pc = if f.z { 0x1400ca344 } else { 0x1400ca316 };
                continue;
            }
            0x1400ca316 => {
                // 1400ca316  66660f1f840000000000  NOP dword ptr [RAX + RAX*0x1]
                pc = 0x1400ca320;
            }
            0x1400ca320 => {
                // 1400ca320  480fbcc2  BSF RAX,RDX
                {
                    let v = r[2];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca324  8905ee4f2300  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 1400ca32a  83c008  ADD EAX,0x8
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x8u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca32d  498b84c080360f00  MOV RAX,qword ptr [R8 + RAX*0x8 + 0xf3680]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3680u64),
                    );
                }
                // 1400ca335  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400ca338  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 1400ca33b  488d42ff  LEA RAX,[RDX + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffffu64);
                }
                // 1400ca33f  4823d0  AND RDX,RAX
                {
                    let v = r[2] & r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400ca342  75dc  JNZ 0x1400ca320
                pc = if !f.z { 0x1400ca320 } else { 0x1400ca344 };
                continue;
            }
            0x1400ca344 => {
                // 1400ca344  8d4610  LEA EAX,[RSI + 0x10]
                {
                    r[0] = (0u64.wrapping_add(r[6]).wrapping_add(0x10u64)) & 0xffff_ffff;
                }
                // 1400ca347  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca34a  4d8b84d080360f00  MOV R8,qword ptr [R8 + RDX*0x8 + 0xf3680]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3680u64),
                    );
                }
                // 1400ca352  4c23c7  AND R8,RDI
                {
                    let v = r[8] & r[7];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400ca355  742d  JZ 0x1400ca384
                pc = if f.z { 0x1400ca384 } else { 0x1400ca357 };
                continue;
            }
            0x1400ca357 => {
                // 1400ca357  488d15a25cf3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca35e  6690  NOP
                pc = 0x1400ca360;
            }
            0x1400ca360 => {
                // 1400ca360  490fbcc0  BSF RAX,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca364  8905ae4f2300  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 1400ca36a  83c0f8  ADD EAX,-0x8
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0xfffffffffffffff8u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca36d  488b84c280400f00  MOV RAX,qword ptr [RDX + RAX*0x8 + 0xf4080]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4080u64),
                    );
                }
                // 1400ca375  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400ca378  4c23d0  AND R10,RAX
                {
                    let v = r[10] & r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 1400ca37b  498d40ff  LEA RAX,[R8 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[8]).wrapping_add(0xffffffffffffffffu64);
                }
                // 1400ca37f  4c23c0  AND R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400ca382  75dc  JNZ 0x1400ca360
                pc = if !f.z { 0x1400ca360 } else { 0x1400ca384 };
                continue;
            }
            0x1400ca384 => {
                // 1400ca384  4d095320  OR qword ptr [R11 + 0x20],R10
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x20u64)) | r[10];
                    f.logic(v, 64);
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x20u64), v);
                }
                // 1400ca388  4c8d05715cf3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca38f  448b9588000000  MOV R10D,dword ptr [RBP + 0x88]
                {
                    r[10] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64))) & 0xffff_ffff;
                }
                pc = 0x1400ca396;
            }
            0x1400ca396 => {
                // 1400ca396  4c6325d36d2f00  MOVSXD R12,dword ptr [0x1403c1170]
                {
                    r[12] =
                        ((m.read::<32>(0u64.wrapping_add(0x1403c1170u64))) as i32 as i64) as u64;
                }
                // 1400ca39d  498bd1  MOV RDX,R9
                {
                    r[2] = r[9];
                }
                // 1400ca3a0  48c1e206  SHL RDX,0x6
                {
                    let v = r[2];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400ca3a4  4a8d0422  LEA RAX,[RDX + R12*0x1]
                {
                    r[0] = 0u64
                        .wrapping_add(r[2])
                        .wrapping_add(r[12].wrapping_mul(0x1u64));
                }
                // 1400ca3a8  460fbe8400707d0f00  MOVSX R8D,byte ptr [RAX + R8*0x1 + 0xf7d70]
                {
                    r[8] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[8].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400ca3b1  453bc2  CMP R8D,R10D
                {
                    f.arithmetic(r[8] & 0xffffffffu64, r[10] & 0xffffffffu64, 32, true);
                }
                // 1400ca3b4  418bc0  MOV EAX,R8D
                {
                    r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca3b7  410f4dc2  CMOVGE EAX,R10D
                {
                    if f.s == f.o {
                        r[0] = (r[10] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 1400ca3bb  448bd0  MOV R10D,EAX
                {
                    r[10] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca3be  898588000000  MOV dword ptr [RBP + 0x88],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x88u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400ca3c4  486305a96d2f00  MOVSXD RAX,dword ptr [0x1403c1174]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 1400ca3cb  4803c2  ADD RAX,RDX
                {
                    let v = f.arithmetic(r[0], r[2], 64, false);
                    r[0] = v;
                }
                // 1400ca3ce  488d152b5cf3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca3d5  0fbe9410707d0f00  MOVSX EDX,byte ptr [RAX + RDX*0x1 + 0xf7d70]
                {
                    r[2] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400ca3dd  413bd7  CMP EDX,R15D
                {
                    f.arithmetic(r[2] & 0xffffffffu64, r[15] & 0xffffffffu64, 32, true);
                }
                // 1400ca3e0  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca3e2  410f4dc7  CMOVGE EAX,R15D
                {
                    if f.s == f.o {
                        r[0] = (r[15] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 1400ca3e6  412bd0  SUB EDX,R8D
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, true);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400ca3e9  4c8b05206d2f00  MOV R8,qword ptr [0x1403c1110]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400ca3f0  03ca  ADD ECX,EDX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca3f2  488d15075cf3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca3f9  898580000000  MOV dword ptr [RBP + 0x80],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x80u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400ca3ff  4c8bbc1380460f00  MOV R15,qword ptr [RBX + RDX*0x1 + 0xf4680]
                {
                    r[15] = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4680u64),
                    );
                }
                // 1400ca407  4d23f8  AND R15,R8
                {
                    let v = r[15] & r[8];
                    f.logic(v, 64);
                    r[15] = v;
                }
                // 1400ca40a  7408  JZ 0x1400ca414
                pc = if f.z { 0x1400ca414 } else { 0x1400ca40c };
                continue;
            }
            0x1400ca40c => {
                // 1400ca40c  42038c8ae0111000  ADD ECX,dword ptr [RDX + R9*0x4 + 0x1011e0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[2])
                                .wrapping_add(r[9].wrapping_mul(0x4u64))
                                .wrapping_add(0x1011e0u64),
                        ),
                        32,
                        false,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca414;
            }
            0x1400ca414 => {
                // 1400ca414  488b8413f0ee0e00  MOV RAX,qword ptr [RBX + RDX*0x1 + 0xeeef0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xeeef0u64),
                    );
                }
                // 1400ca41c  480b8413f0f00e00  OR RAX,qword ptr [RBX + RDX*0x1 + 0xef0f0]
                {
                    let v = r[0]
                        | m.read::<64>(
                            0u64.wrapping_add(r[3])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0xef0f0u64),
                        );
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca424  4985c0  TEST R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                }
                // 1400ca427  7408  JZ 0x1400ca431
                pc = if f.z { 0x1400ca431 } else { 0x1400ca429 };
                continue;
            }
            0x1400ca429 => {
                // 1400ca429  42038c8ae0101000  ADD ECX,dword ptr [RDX + R9*0x4 + 0x1010e0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[2])
                                .wrapping_add(r[9].wrapping_mul(0x4u64))
                                .wrapping_add(0x1010e0u64),
                        ),
                        32,
                        false,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca431;
            }
            0x1400ca431 => {
                // 1400ca431  488b3de06c2f00  MOV RDI,qword ptr [0x1403c1118]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400ca438  498bc0  MOV RAX,R8
                {
                    r[0] = r[8];
                }
                // 1400ca43b  480bc7  OR RAX,RDI
                {
                    let v = r[0] | r[7];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca43e  4885841380440f00  TEST qword ptr [RBX + RDX*0x1 + 0xf4480],RAX
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4480u64),
                    ) & r[0];
                    f.logic(v, 64);
                }
                // 1400ca446  0f8413010000  JZ 0x1400ca55f
                pc = if f.z { 0x1400ca55f } else { 0x1400ca44c };
                continue;
            }
            0x1400ca44c => {
                // 1400ca44c  4c858413f0e80e00  TEST qword ptr [RBX + RDX*0x1 + 0xee8f0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xee8f0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca454  741c  JZ 0x1400ca472
                pc = if f.z { 0x1400ca472 } else { 0x1400ca456 };
                continue;
            }
            0x1400ca456 => {
                // 1400ca456  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 1400ca459  81e932001400  SUB ECX,0x140032
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x140032u64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca45f  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca462  4c8584c2d02a1100  TEST qword ptr [RDX + RAX*0x8 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca46a  7506  JNZ 0x1400ca472
                pc = if !f.z { 0x1400ca472 } else { 0x1400ca46c };
                continue;
            }
            0x1400ca46c => {
                // 1400ca46c  81e920001800  SUB ECX,0x180020
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x180020u64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca472;
            }
            0x1400ca472 => {
                // 1400ca472  4183e107  AND R9D,0x7
                {
                    let v = (r[9] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400ca476  4e8584cad02a1100  TEST qword ptr [RDX + R9*0x8 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[9].wrapping_mul(0x8u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca47e  750b  JNZ 0x1400ca48b
                pc = if !f.z { 0x1400ca48b } else { 0x1400ca480 };
                continue;
            }
            0x1400ca480 => {
                // 1400ca480  81e938002a00  SUB ECX,0x2a0038
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x2a0038u64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca486  e9a0000000  JMP 0x1400ca52b
                pc = 0x1400ca52b;
                continue;
            }
            0x1400ca48b => {
                // 1400ca48b  4c85841380380f00  TEST qword ptr [RBX + RDX*0x1 + 0xf3880],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3880u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca493  0f8592000000  JNZ 0x1400ca52b
                pc = if !f.z { 0x1400ca52b } else { 0x1400ca499 };
                continue;
            }
            0x1400ca499 => {
                // 1400ca499  4885bc13803c0f00  TEST qword ptr [RBX + RDX*0x1 + 0xf3c80],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3c80u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca4a1  4c8d3d585bf3ff  LEA R15,[0x140000000]
                {
                    r[15] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca4a8  41ba08000000  MOV R10D,0x8
                {
                    r[10] = (0x8u64) & 0xffff_ffff;
                }
                // 1400ca4ae  410f95c1  SETNZ R9B
                {
                    r[9] = (r[9] & !0xffu64) | ((u64::from(!f.z)) & 0xffu64);
                }
                // 1400ca4b2  4c85841380480f00  TEST qword ptr [RBX + RDX*0x1 + 0xf4880],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4880u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca4ba  754c  JNZ 0x1400ca508
                pc = if !f.z { 0x1400ca508 } else { 0x1400ca4bc };
                continue;
            }
            0x1400ca4bc => {
                // 1400ca4bc  8d4608  LEA EAX,[RSI + 0x8]
                {
                    r[0] = (0u64.wrapping_add(r[6]).wrapping_add(0x8u64)) & 0xffff_ffff;
                }
                // 1400ca4bf  41ba10000000  MOV R10D,0x10
                {
                    r[10] = (0x10u64) & 0xffff_ffff;
                }
                // 1400ca4c5  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca4c8  4985bcd7803c0f00  TEST qword ptr [R15 + RDX*0x8 + 0xf3c80],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3c80u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca4d0  0f95c0  SETNZ AL
                {
                    r[0] = (r[0] & !0xffu64) | ((u64::from(!f.z)) & 0xffu64);
                }
                // 1400ca4d3  410ac1  OR AL,R9B
                {
                    let v = (r[0] & 0xffu64 | r[9] & 0xffu64) & 0xffu64;
                    f.logic(v, 8);
                    r[0] = (r[0] & !0xffu64) | ((v) & 0xffu64);
                }
                // 1400ca4d6  440fb6c8  MOVZX R9D,AL
                {
                    r[9] = (r[0] & 0xffu64) & 0xffff_ffff;
                }
                // 1400ca4da  0fb6d8  MOVZX EBX,AL
                {
                    r[3] = (r[0] & 0xffu64) & 0xffff_ffff;
                }
                // 1400ca4dd  4d8584d780480f00  TEST qword ptr [R15 + RDX*0x8 + 0xf4880],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4880u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca4e5  7521  JNZ 0x1400ca508
                pc = if !f.z { 0x1400ca508 } else { 0x1400ca4e7 };
                continue;
            }
            0x1400ca4e7 => {
                // 1400ca4e7  8d4610  LEA EAX,[RSI + 0x10]
                {
                    r[0] = (0u64.wrapping_add(r[6]).wrapping_add(0x10u64)) & 0xffff_ffff;
                }
                // 1400ca4ea  440fb6cb  MOVZX R9D,BL
                {
                    r[9] = (r[3] & 0xffu64) & 0xffff_ffff;
                }
                // 1400ca4ee  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca4f1  41ba18000000  MOV R10D,0x18
                {
                    r[10] = (0x18u64) & 0xffff_ffff;
                }
                // 1400ca4f7  b801000000  MOV EAX,0x1
                {
                    r[0] = (0x1u64) & 0xffff_ffff;
                }
                // 1400ca4fc  4985bcd7803c0f00  TEST qword ptr [R15 + RDX*0x8 + 0xf3c80],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3c80u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca504  440f45c8  CMOVNZ R9D,EAX
                {
                    if !f.z {
                        r[9] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                pc = 0x1400ca508;
            }
            0x1400ca508 => {
                // 1400ca508  4584c9  TEST R9B,R9B
                {
                    let v = (r[9] & 0xffu64) & 0xffu64;
                    f.logic(v, 8);
                }
                // 1400ca50b  7511  JNZ 0x1400ca51e
                pc = if !f.z { 0x1400ca51e } else { 0x1400ca50d };
                continue;
            }
            0x1400ca50d => {
                // 1400ca50d  418d0432  LEA EAX,[R10 + RSI*0x1]
                {
                    r[0] = (0u64
                        .wrapping_add(r[10])
                        .wrapping_add(r[6].wrapping_mul(0x1u64)))
                        & 0xffff_ffff;
                }
                // 1400ca511  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca514  4985bcd780480f00  TEST qword ptr [R15 + RDX*0x8 + 0xf4880],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[15])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4880u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca51c  7406  JZ 0x1400ca524
                pc = if f.z { 0x1400ca524 } else { 0x1400ca51e };
                continue;
            }
            0x1400ca51e => {
                // 1400ca51e  81e934002700  SUB ECX,0x270034
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x270034u64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca524;
            }
            0x1400ca524 => {
                // 1400ca524  448b9588000000  MOV R10D,dword ptr [RBP + 0x88]
                {
                    r[10] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64))) & 0xffff_ffff;
                }
                pc = 0x1400ca52b;
            }
            0x1400ca52b => {
                // 1400ca52b  4c8d05ce5af3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                pc = 0x1400ca532;
            }
            0x1400ca532 => {
                // 1400ca532  448bbd80000000  MOV R15D,dword ptr [RBP + 0x80]
                {
                    r[15] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x80u64))) & 0xffff_ffff;
                }
                // 1400ca539  498d45ff  LEA RAX,[R13 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[13]).wrapping_add(0xffffffffffffffffu64);
                }
                // 1400ca53d  4c23e8  AND R13,RAX
                {
                    let v = r[13] & r[0];
                    f.logic(v, 64);
                    r[13] = v;
                }
                // 1400ca540  0f858afdffff  JNZ 0x1400ca2d0
                pc = if !f.z { 0x1400ca2d0 } else { 0x1400ca546 };
                continue;
            }
            0x1400ca546 => {
                // 1400ca546  44899588000000  MOV dword ptr [RBP + 0x88],R10D
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x88u64),
                        r[10] & 0xffffffffu64,
                    );
                }
                // 1400ca54d  458bcf  MOV R9D,R15D
                {
                    r[9] = (r[15] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca550  4489bd80000000  MOV dword ptr [RBP + 0x80],R15D
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x80u64),
                        r[15] & 0xffffffffu64,
                    );
                }
                // 1400ca557  4533e4  XOR R12D,R12D
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[12] = (v) & 0xffff_ffff;
                }
                // 1400ca55a  e971020000  JMP 0x1400ca7d0
                pc = 0x1400ca7d0;
                continue;
            }
            0x1400ca55f => {
                // 1400ca55f  4c858413f0e80e00  TEST qword ptr [RBX + RDX*0x1 + 0xee8f0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xee8f0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca567  741c  JZ 0x1400ca585
                pc = if f.z { 0x1400ca585 } else { 0x1400ca569 };
                continue;
            }
            0x1400ca569 => {
                // 1400ca569  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 1400ca56c  81e932001400  SUB ECX,0x140032
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x140032u64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca572  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca575  4c8584c2d02a1100  TEST qword ptr [RDX + RAX*0x8 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca57d  7506  JNZ 0x1400ca585
                pc = if !f.z { 0x1400ca585 } else { 0x1400ca57f };
                continue;
            }
            0x1400ca57f => {
                // 1400ca57f  81e96e005200  SUB ECX,0x52006e
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x52006eu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca585;
            }
            0x1400ca585 => {
                // 1400ca585  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 1400ca588  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca58b  4c8d14c500000000  LEA R10,[RAX*0x8]
                {
                    r[10] = 0u64.wrapping_add(r[0].wrapping_mul(0x8u64));
                }
                // 1400ca593  4d858412d02a1100  TEST qword ptr [R10 + RDX*0x1 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca59b  750a  JNZ 0x1400ca5a7
                pc = if !f.z { 0x1400ca5a7 } else { 0x1400ca59d };
                continue;
            }
            0x1400ca59d => {
                // 1400ca59d  422b8c8ae00f1000  SUB ECX,dword ptr [RDX + R9*0x4 + 0x100fe0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[2])
                                .wrapping_add(r[9].wrapping_mul(0x4u64))
                                .wrapping_add(0x100fe0u64),
                        ),
                        32,
                        true,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca5a5  eb10  JMP 0x1400ca5b7
                pc = 0x1400ca5b7;
                continue;
            }
            0x1400ca5a7 => {
                // 1400ca5a7  4c85841380380f00  TEST qword ptr [RBX + RDX*0x1 + 0xf3880],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3880u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca5af  7506  JNZ 0x1400ca5b7
                pc = if !f.z { 0x1400ca5b7 } else { 0x1400ca5b1 };
                continue;
            }
            0x1400ca5b1 => {
                // 1400ca5b1  81e978005a00  SUB ECX,0x5a0078
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x5a0078u64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca5b7;
            }
            0x1400ca5b7 => {
                // 1400ca5b7  4c8bb41380340f00  MOV R14,qword ptr [RBX + RDX*0x1 + 0xf3480]
                {
                    r[14] = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3480u64),
                    );
                }
                // 1400ca5bf  4c23f7  AND R14,RDI
                {
                    let v = r[14] & r[7];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 1400ca5c2  0f8549010000  JNZ 0x1400ca711
                pc = if !f.z { 0x1400ca711 } else { 0x1400ca5c8 };
                continue;
            }
            0x1400ca5c8 => {
                // 1400ca5c8  49c1e107  SHL R9,0x7
                {
                    let v = r[9];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400ca5cc  41038c11e02a1000  ADD ECX,dword ptr [R9 + RDX*0x1 + 0x102ae0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[9])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0x102ae0u64),
                        ),
                        32,
                        false,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca5d4  4985bc12902a1100  TEST qword ptr [R10 + RDX*0x1 + 0x112a90],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x112a90u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca5dc  740a  JZ 0x1400ca5e8
                pc = if f.z { 0x1400ca5e8 } else { 0x1400ca5de };
                continue;
            }
            0x1400ca5de => {
                // 1400ca5de  4985bc12502a1100  TEST qword ptr [R10 + RDX*0x1 + 0x112a50],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x112a50u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca5e6  7508  JNZ 0x1400ca5f0
                pc = if !f.z { 0x1400ca5f0 } else { 0x1400ca5e8 };
                continue;
            }
            0x1400ca5e8 => {
                // 1400ca5e8  41038c11e42a1000  ADD ECX,dword ptr [R9 + RDX*0x1 + 0x102ae4]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[9])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0x102ae4u64),
                        ),
                        32,
                        false,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca5f0;
            }
            0x1400ca5f0 => {
                // 1400ca5f0  4d85ff  TEST R15,R15
                {
                    let v = r[15];
                    f.logic(v, 64);
                }
                // 1400ca5f3  7408  JZ 0x1400ca5fd
                pc = if f.z { 0x1400ca5fd } else { 0x1400ca5f5 };
                continue;
            }
            0x1400ca5f5 => {
                // 1400ca5f5  41038c11e82a1000  ADD ECX,dword ptr [R9 + RDX*0x1 + 0x102ae8]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[9])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0x102ae8u64),
                        ),
                        32,
                        false,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca5fd;
            }
            0x1400ca5fd => {
                // 1400ca5fd  4c8b8413f0ea0e00  MOV R8,qword ptr [RBX + RDX*0x1 + 0xeeaf0]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xeeaf0u64),
                    );
                }
                // 1400ca605  488d1df459f3ff  LEA RBX,[0x140000000]
                {
                    r[3] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca60c  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400ca613  498bd0  MOV RDX,R8
                {
                    r[2] = r[8];
                }
                // 1400ca616  4923d6  AND RDX,R14
                {
                    let v = r[2] & r[14];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400ca619  744a  JZ 0x1400ca665
                pc = if f.z { 0x1400ca665 } else { 0x1400ca61b };
                continue;
            }
            0x1400ca61b => {
                // 1400ca61b  458b9419ec2a1000  MOV R10D,dword ptr [R9 + RBX*0x1 + 0x102aec]
                {
                    r[10] = (m.read::<32>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[3].wrapping_mul(0x1u64))
                            .wrapping_add(0x102aecu64),
                    )) & 0xffff_ffff;
                }
                // 1400ca623  480fbcc2  BSF RAX,RDX
                {
                    let v = r[2];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca627  8905eb4c2300  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 1400ca62d  48c1e007  SHL RAX,0x7
                {
                    let v = r[0];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400ca631  8b8418ec2a1000  MOV EAX,dword ptr [RAX + RBX*0x1 + 0x102aec]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[3].wrapping_mul(0x1u64))
                            .wrapping_add(0x102aecu64),
                    )) & 0xffff_ffff;
                }
                // 1400ca638  4103c2  ADD EAX,R10D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[10] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca63b  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca63d  488d42ff  LEA RAX,[RDX + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffffu64);
                }
                // 1400ca641  4823c2  AND RAX,RDX
                {
                    let v = r[0] & r[2];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca644  4c23c0  AND R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400ca647  741c  JZ 0x1400ca665
                pc = if f.z { 0x1400ca665 } else { 0x1400ca649 };
                continue;
            }
            0x1400ca649 => {
                // 1400ca649  490fbcc0  BSF RAX,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca64d  8bd0  MOV EDX,EAX
                {
                    r[2] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca64f  48c1e207  SHL RDX,0x7
                {
                    let v = r[2];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400ca653  8905bf4c2300  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 1400ca659  8b841aec2a1000  MOV EAX,dword ptr [RDX + RBX*0x1 + 0x102aec]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[3].wrapping_mul(0x1u64))
                            .wrapping_add(0x102aecu64),
                    )) & 0xffff_ffff;
                }
                // 1400ca660  4103c2  ADD EAX,R10D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[10] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca663  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca665;
            }
            0x1400ca665 => {
                // 1400ca665  498bc5  MOV RAX,R13
                {
                    r[0] = r[13];
                }
                // 1400ca668  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 1400ca66b  4923c5  AND RAX,R13
                {
                    let v = r[0] & r[13];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca66e  4c0bf0  OR R14,RAX
                {
                    let v = r[14] | r[0];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 1400ca671  4c89b590000000  MOV qword ptr [RBP + 0x90],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64), r[14]);
                }
                // 1400ca678  83fe18  CMP ESI,0x18
                {
                    f.arithmetic(r[6] & 0xffffffffu64, 0x18u64, 32, true);
                }
                // 1400ca67b  0f8ca3feffff  JL 0x1400ca524
                pc = if f.s != f.o { 0x1400ca524 } else { 0x1400ca681 };
                continue;
            }
            0x1400ca681 => {
                // 1400ca681  b818000000  MOV EAX,0x18
                {
                    r[0] = (0x18u64) & 0xffff_ffff;
                }
                // 1400ca686  3bf0  CMP ESI,EAX
                {
                    f.arithmetic(r[6] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                }
                // 1400ca688  0f4fc6  CMOVG EAX,ESI
                {
                    if !f.z && f.s == f.o {
                        r[0] = (r[6] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 1400ca68b  83e818  SUB EAX,0x18
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x18u64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca68e  4c63d0  MOVSXD R10,EAX
                {
                    r[10] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca691  8d4608  LEA EAX,[RSI + 0x8]
                {
                    r[0] = (0u64.wrapping_add(r[6]).wrapping_add(0x8u64)) & 0xffff_ffff;
                }
                // 1400ca694  4c63c0  MOVSXD R8,EAX
                {
                    r[8] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca697  486305d66a2f00  MOVSXD RAX,dword ptr [0x1403c1174]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 1400ca69e  49c1e006  SHL R8,0x6
                {
                    let v = r[8];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400ca6a2  4903c0  ADD RAX,R8
                {
                    let v = f.arithmetic(r[0], r[8], 64, false);
                    r[0] = v;
                }
                // 1400ca6a5  49c1e206  SHL R10,0x6
                {
                    let v = r[10];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[10] = v;
                    }
                }
                // 1400ca6a9  0fbe9418707d0f00  MOVSX EDX,byte ptr [RAX + RBX*0x1 + 0xf7d70]
                {
                    r[2] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[3].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400ca6b1  b867666666  MOV EAX,0x66666667
                {
                    r[0] = (0x66666667u64) & 0xffff_ffff;
                }
                // 1400ca6b6  410faf941afc4a1000  IMUL EDX,dword ptr [R10 + RBX*0x1 + 0x104afc]
                {
                    r[2] = ((r[2] & 0xffffffffu64).wrapping_mul(
                        m.read::<32>(
                            0u64.wrapping_add(r[10])
                                .wrapping_add(r[3].wrapping_mul(0x1u64))
                                .wrapping_add(0x104afcu64),
                        ),
                    )) & 0xffff_ffff;
                }
                // 1400ca6bf  f7ea  IMUL EDX
                {
                    let v = (r[0] as i32 as i128) * ((r[2] & 0xffffffffu64) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 1400ca6c1  448bca  MOV R9D,EDX
                {
                    r[9] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca6c4  41c1f902  SAR R9D,0x2
                {
                    let v = r[9] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[9] = (v) & 0xffff_ffff;
                    }
                }
                // 1400ca6c8  418bc1  MOV EAX,R9D
                {
                    r[0] = (r[9] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca6cb  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400ca6ce  4403c8  ADD R9D,EAX
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400ca6d1  4b8d0404  LEA RAX,[R12 + R8*0x1]
                {
                    r[0] = 0u64
                        .wrapping_add(r[12])
                        .wrapping_add(r[8].wrapping_mul(0x1u64));
                }
                // 1400ca6d5  4c8d052459f3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca6dc  420fbe9400707d0f00  MOVSX EDX,byte ptr [RAX + R8*0x1 + 0xf7d70]
                {
                    r[2] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[8].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400ca6e5  b867666666  MOV EAX,0x66666667
                {
                    r[0] = (0x66666667u64) & 0xffff_ffff;
                }
                // 1400ca6ea  430faf9402f84a1000  IMUL EDX,dword ptr [R10 + R8*0x1 + 0x104af8]
                {
                    r[2] = ((r[2] & 0xffffffffu64).wrapping_mul(
                        m.read::<32>(
                            0u64.wrapping_add(r[10])
                                .wrapping_add(r[8].wrapping_mul(0x1u64))
                                .wrapping_add(0x104af8u64),
                        ),
                    )) & 0xffff_ffff;
                }
                // 1400ca6f3  448b9588000000  MOV R10D,dword ptr [RBP + 0x88]
                {
                    r[10] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64))) & 0xffff_ffff;
                }
                // 1400ca6fa  f7ea  IMUL EDX
                {
                    let v = (r[0] as i32 as i128) * ((r[2] & 0xffffffffu64) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 1400ca6fc  c1fa02  SAR EDX,0x2
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 1400ca6ff  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca701  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400ca704  03d0  ADD EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400ca706  442bca  SUB R9D,EDX
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400ca709  4103c9  ADD ECX,R9D
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[9] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca70c  e921feffff  JMP 0x1400ca532
                pc = 0x1400ca532;
                continue;
            }
            0x1400ca711 => {
                // 1400ca711  488b841380380f00  MOV RAX,qword ptr [RBX + RDX*0x1 + 0xf3880]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[3])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3880u64),
                    );
                }
                // 1400ca719  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca71c  f3480fb8d0  POPCNT RDX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[2] = u64::from(v.count_ones());
                }
                // 1400ca721  f3490fb8c6  POPCNT RAX,R14
                {
                    let v = r[14];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 1400ca726  3bc2  CMP EAX,EDX
                {
                    f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                }
                // 1400ca728  7f7d  JG 0x1400ca7a7
                pc = if !f.z && f.s == f.o {
                    0x1400ca7a7
                } else {
                    0x1400ca72a
                };
                continue;
            }
            0x1400ca72a => {
                // 1400ca72a  488d15cf58f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca731  49c1e107  SHL R9,0x7
                {
                    let v = r[9];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400ca735  418b8411e02a1000  MOV EAX,dword ptr [R9 + RDX*0x1 + 0x102ae0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x102ae0u64),
                    )) & 0xffff_ffff;
                }
                // 1400ca73d  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400ca73e  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca740  488d15b958f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca747  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400ca749  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca74b  4985bc12902a1100  TEST qword ptr [R10 + RDX*0x1 + 0x112a90],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x112a90u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca753  740a  JZ 0x1400ca75f
                pc = if f.z { 0x1400ca75f } else { 0x1400ca755 };
                continue;
            }
            0x1400ca755 => {
                // 1400ca755  4985bc12502a1100  TEST qword ptr [R10 + RDX*0x1 + 0x112a50],RDI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x112a50u64),
                    ) & r[7];
                    f.logic(v, 64);
                }
                // 1400ca75d  7516  JNZ 0x1400ca775
                pc = if !f.z { 0x1400ca775 } else { 0x1400ca75f };
                continue;
            }
            0x1400ca75f => {
                // 1400ca75f  418b8411e42a1000  MOV EAX,dword ptr [R9 + RDX*0x1 + 0x102ae4]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x102ae4u64),
                    )) & 0xffff_ffff;
                }
                // 1400ca767  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400ca768  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca76a  488d158f58f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca771  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400ca773  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca775;
            }
            0x1400ca775 => {
                // 1400ca775  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400ca77c  4c8d057d58f3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca783  448b9588000000  MOV R10D,dword ptr [RBP + 0x88]
                {
                    r[10] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64))) & 0xffff_ffff;
                }
                // 1400ca78a  4d85ff  TEST R15,R15
                {
                    let v = r[15];
                    f.logic(v, 64);
                }
                // 1400ca78d  0f849ffdffff  JZ 0x1400ca532
                pc = if f.z { 0x1400ca532 } else { 0x1400ca793 };
                continue;
            }
            0x1400ca793 => {
                // 1400ca793  418b8411e82a1000  MOV EAX,dword ptr [R9 + RDX*0x1 + 0x102ae8]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0x102ae8u64),
                    )) & 0xffff_ffff;
                }
                // 1400ca79b  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400ca79c  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca79e  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400ca7a0  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca7a2  e98bfdffff  JMP 0x1400ca532
                pc = 0x1400ca532;
                continue;
            }
            0x1400ca7a7 => {
                // 1400ca7a7  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400ca7ae  4c8d054b58f3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca7b5  448b9588000000  MOV R10D,dword ptr [RBP + 0x88]
                {
                    r[10] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64))) & 0xffff_ffff;
                }
                // 1400ca7bc  49c1e105  SHL R9,0x5
                {
                    let v = r[9];
                    let n = 5u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400ca7c0  43038c01e0221000  ADD ECX,dword ptr [R9 + R8*0x1 + 0x1022e0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[9])
                                .wrapping_add(r[8].wrapping_mul(0x1u64))
                                .wrapping_add(0x1022e0u64),
                        ),
                        32,
                        false,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca7c8  e965fdffff  JMP 0x1400ca532
                pc = 0x1400ca532;
                continue;
            }
            0x1400ca7cd => {
                // 1400ca7cd  458bca  MOV R9D,R10D
                {
                    r[9] = (r[10] & 0xffffffffu64) & 0xffff_ffff;
                }
                pc = 0x1400ca7d0;
            }
            0x1400ca7d0 => {
                // 1400ca7d0  4d8933  MOV qword ptr [R11],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[11]), r[14]);
                }
                // 1400ca7d3  4d8bf4  MOV R14,R12
                {
                    r[14] = r[12];
                }
                // 1400ca7d6  4c8b2d3b692f00  MOV R13,qword ptr [0x1403c1118]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400ca7dd  4c89a590000000  MOV qword ptr [RBP + 0x90],R12
                {
                    m.write::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64), r[12]);
                }
                // 1400ca7e4  4d85ed  TEST R13,R13
                {
                    let v = r[13];
                    f.logic(v, 64);
                }
                // 1400ca7e7  0f84d3020000  JZ 0x1400caac0
                pc = if f.z { 0x1400caac0 } else { 0x1400ca7ed };
                continue;
            }
            0x1400ca7ed => {
                // 1400ca7ed  488b351c692f00  MOV RSI,qword ptr [0x1403c1110]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400ca7f4  0f1f4000  NOP dword ptr [RAX]
                // 1400ca7f8  0f1f840000000000  NOP dword ptr [RAX + RAX*0x1]
                pc = 0x1400ca800;
            }
            0x1400ca800 => {
                // 1400ca800  490fbcdd  BSF RBX,R13
                {
                    let v = r[13];
                    f.z = v == 0;
                    if v != 0 {
                        r[3] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca804  4863fb  MOVSXD RDI,EBX
                {
                    r[7] = ((r[3] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca807  891d0b4b2300  MOV dword ptr [0x1402ff318],EBX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[3] & 0xffffffffu64);
                }
                // 1400ca80d  4c8d14fd00000000  LEA R10,[RDI*0x8]
                {
                    r[10] = 0u64.wrapping_add(r[7].wrapping_mul(0x8u64));
                }
                // 1400ca815  83fb10  CMP EBX,0x10
                {
                    f.arithmetic(r[3] & 0xffffffffu64, 0x10u64, 32, true);
                }
                // 1400ca818  7d11  JGE 0x1400ca82b
                pc = if f.s == f.o { 0x1400ca82b } else { 0x1400ca81a };
                continue;
            }
            0x1400ca81a => {
                // 1400ca81a  4b8b840280460f00  MOV RAX,qword ptr [R10 + R8*0x1 + 0xf4680]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[8].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4680u64),
                    );
                }
                // 1400ca822  49094328  OR qword ptr [R11 + 0x28],RAX
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x28u64)) | r[0];
                    f.logic(v, 64);
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x28u64), v);
                }
                // 1400ca826  e99b000000  JMP 0x1400ca8c6
                pc = 0x1400ca8c6;
                continue;
            }
            0x1400ca82b => {
                // 1400ca82b  8d43f8  LEA EAX,[RBX + -0x8]
                {
                    r[0] =
                        (0u64.wrapping_add(r[3]).wrapping_add(0xfffffffffffffff8u64)) & 0xffff_ffff;
                }
                // 1400ca82e  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca831  4d8b8cd080380f00  MOV R9,qword ptr [R8 + RDX*0x8 + 0xf3880]
                {
                    r[9] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3880u64),
                    );
                }
                // 1400ca839  4b8b940280420f00  MOV RDX,qword ptr [R10 + R8*0x1 + 0xf4280]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[8].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4280u64),
                    );
                }
                // 1400ca841  4823d6  AND RDX,RSI
                {
                    let v = r[2] & r[6];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400ca844  742e  JZ 0x1400ca874
                pc = if f.z { 0x1400ca874 } else { 0x1400ca846 };
                continue;
            }
            0x1400ca846 => {
                // 1400ca846  66660f1f840000000000  NOP dword ptr [RAX + RAX*0x1]
                pc = 0x1400ca850;
            }
            0x1400ca850 => {
                // 1400ca850  480fbcc2  BSF RAX,RDX
                {
                    let v = r[2];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca854  8905be4a2300  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 1400ca85a  83c0f8  ADD EAX,-0x8
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0xfffffffffffffff8u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca85d  498b84c080380f00  MOV RAX,qword ptr [R8 + RAX*0x8 + 0xf3880]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3880u64),
                    );
                }
                // 1400ca865  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400ca868  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400ca86b  488d42ff  LEA RAX,[RDX + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[2]).wrapping_add(0xffffffffffffffffu64);
                }
                // 1400ca86f  4823d0  AND RDX,RAX
                {
                    let v = r[2] & r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400ca872  75dc  JNZ 0x1400ca850
                pc = if !f.z { 0x1400ca850 } else { 0x1400ca874 };
                continue;
            }
            0x1400ca874 => {
                // 1400ca874  8d43f0  LEA EAX,[RBX + -0x10]
                {
                    r[0] =
                        (0u64.wrapping_add(r[3]).wrapping_add(0xfffffffffffffff0u64)) & 0xffff_ffff;
                }
                // 1400ca877  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca87a  4d8b84d080380f00  MOV R8,qword ptr [R8 + RDX*0x8 + 0xf3880]
                {
                    r[8] = m.read::<64>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3880u64),
                    );
                }
                // 1400ca882  4c23c6  AND R8,RSI
                {
                    let v = r[8] & r[6];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400ca885  742d  JZ 0x1400ca8b4
                pc = if f.z { 0x1400ca8b4 } else { 0x1400ca887 };
                continue;
            }
            0x1400ca887 => {
                // 1400ca887  488d157257f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca88e  6690  NOP
                pc = 0x1400ca890;
            }
            0x1400ca890 => {
                // 1400ca890  490fbcc0  BSF RAX,R8
                {
                    let v = r[8];
                    f.z = v == 0;
                    if v != 0 {
                        r[0] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400ca894  89057e4a2300  MOV dword ptr [0x1402ff318],EAX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[0] & 0xffffffffu64);
                }
                // 1400ca89a  83c008  ADD EAX,0x8
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x8u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca89d  488b84c2803e0f00  MOV RAX,qword ptr [RDX + RAX*0x8 + 0xf3e80]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3e80u64),
                    );
                }
                // 1400ca8a5  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400ca8a8  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400ca8ab  498d40ff  LEA RAX,[R8 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[8]).wrapping_add(0xffffffffffffffffu64);
                }
                // 1400ca8af  4c23c0  AND R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400ca8b2  75dc  JNZ 0x1400ca890
                pc = if !f.z { 0x1400ca890 } else { 0x1400ca8b4 };
                continue;
            }
            0x1400ca8b4 => {
                // 1400ca8b4  4d094b28  OR qword ptr [R11 + 0x28],R9
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x28u64)) | r[9];
                    f.logic(v, 64);
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x28u64), v);
                }
                // 1400ca8b8  4c8d054157f3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca8bf  448b8d80000000  MOV R9D,dword ptr [RBP + 0x80]
                {
                    r[9] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x80u64))) & 0xffff_ffff;
                }
                pc = 0x1400ca8c6;
            }
            0x1400ca8c6 => {
                // 1400ca8c6  4c633da7682f00  MOVSXD R15,dword ptr [0x1403c1174]
                {
                    r[15] =
                        ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 1400ca8cd  488d152c57f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca8d4  4c632595682f00  MOVSXD R12,dword ptr [0x1403c1170]
                {
                    r[12] =
                        ((m.read::<32>(0u64.wrapping_add(0x1403c1170u64))) as i32 as i64) as u64;
                }
                // 1400ca8db  49c1e706  SHL R15,0x6
                {
                    let v = r[15];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[15] = v;
                    }
                }
                // 1400ca8df  498d043f  LEA RAX,[R15 + RDI*0x1]
                {
                    r[0] = 0u64
                        .wrapping_add(r[15])
                        .wrapping_add(r[7].wrapping_mul(0x1u64));
                }
                // 1400ca8e3  460fbe8400707d0f00  MOVSX R8D,byte ptr [RAX + R8*0x1 + 0xf7d70]
                {
                    r[8] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[8].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400ca8ec  453bc1  CMP R8D,R9D
                {
                    f.arithmetic(r[8] & 0xffffffffu64, r[9] & 0xffffffffu64, 32, true);
                }
                // 1400ca8ef  418bc0  MOV EAX,R8D
                {
                    r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca8f2  410f4dc1  CMOVGE EAX,R9D
                {
                    if f.s == f.o {
                        r[0] = (r[9] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 1400ca8f6  49c1e406  SHL R12,0x6
                {
                    let v = r[12];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[12] = v;
                    }
                }
                // 1400ca8fa  448bc8  MOV R9D,EAX
                {
                    r[9] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca8fd  898580000000  MOV dword ptr [RBP + 0x80],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x80u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400ca903  498d043c  LEA RAX,[R12 + RDI*0x1]
                {
                    r[0] = 0u64
                        .wrapping_add(r[12])
                        .wrapping_add(r[7].wrapping_mul(0x1u64));
                }
                // 1400ca907  0fbe9410707d0f00  MOVSX EDX,byte ptr [RAX + RDX*0x1 + 0xf7d70]
                {
                    r[2] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400ca90f  3b9588000000  CMP EDX,dword ptr [RBP + 0x88]
                {
                    f.arithmetic(
                        r[2] & 0xffffffffu64,
                        m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64)),
                        32,
                        true,
                    );
                }
                // 1400ca915  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400ca917  0f4d8588000000  CMOVGE EAX,dword ptr [RBP + 0x88]
                {
                    if f.s == f.o {
                        r[0] = (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64)))
                            & 0xffff_ffff;
                    }
                }
                // 1400ca91e  442bc2  SUB R8D,EDX
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 1400ca921  4103c8  ADD ECX,R8D
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca924  898588000000  MOV dword ptr [RBP + 0x88],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x88u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400ca92a  4c8b05e7672f00  MOV R8,qword ptr [0x1403c1118]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400ca931  488d15c856f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca938  4d8bb41280480f00  MOV R14,qword ptr [R10 + RDX*0x1 + 0xf4880]
                {
                    r[14] = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4880u64),
                    );
                }
                // 1400ca940  4d23f0  AND R14,R8
                {
                    let v = r[14] & r[8];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 1400ca943  741f  JZ 0x1400ca964
                pc = if f.z { 0x1400ca964 } else { 0x1400ca945 };
                continue;
            }
            0x1400ca945 => {
                // 1400ca945  b83f000000  MOV EAX,0x3f
                {
                    r[0] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400ca94a  2bc3  SUB EAX,EBX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[3] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca94c  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca94f  488d05aa56f3ff  LEA RAX,[0x140000000]
                {
                    r[0] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca956  2b8c90e0111000  SUB ECX,dword ptr [RAX + RDX*0x4 + 0x1011e0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[0])
                                .wrapping_add(r[2].wrapping_mul(0x4u64))
                                .wrapping_add(0x1011e0u64),
                        ),
                        32,
                        true,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca95d  488d159c56f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                pc = 0x1400ca964;
            }
            0x1400ca964 => {
                // 1400ca964  498b8412f0ee0e00  MOV RAX,qword ptr [R10 + RDX*0x1 + 0xeeef0]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xeeef0u64),
                    );
                }
                // 1400ca96c  490b8412f0f00e00  OR RAX,qword ptr [R10 + RDX*0x1 + 0xef0f0]
                {
                    let v = r[0]
                        | m.read::<64>(
                            0u64.wrapping_add(r[10])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0xef0f0u64),
                        );
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca974  4985c0  TEST R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                }
                // 1400ca977  741f  JZ 0x1400ca998
                pc = if f.z { 0x1400ca998 } else { 0x1400ca979 };
                continue;
            }
            0x1400ca979 => {
                // 1400ca979  b83f000000  MOV EAX,0x3f
                {
                    r[0] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400ca97e  2bc3  SUB EAX,EBX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[3] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca980  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400ca983  488d057656f3ff  LEA RAX,[0x140000000]
                {
                    r[0] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400ca98a  2b8c90e0101000  SUB ECX,dword ptr [RAX + RDX*0x4 + 0x1010e0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[0])
                                .wrapping_add(r[2].wrapping_mul(0x4u64))
                                .wrapping_add(0x1010e0u64),
                        ),
                        32,
                        true,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca991  488d156856f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                pc = 0x1400ca998;
            }
            0x1400ca998 => {
                // 1400ca998  488b3571672f00  MOV RSI,qword ptr [0x1403c1110]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400ca99f  488bc6  MOV RAX,RSI
                {
                    r[0] = r[6];
                }
                // 1400ca9a2  490bc0  OR RAX,R8
                {
                    let v = r[0] | r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400ca9a5  4985841280420f00  TEST qword ptr [R10 + RDX*0x1 + 0xf4280],RAX
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4280u64),
                    ) & r[0];
                    f.logic(v, 64);
                }
                // 1400ca9ad  0f8412020000  JZ 0x1400cabc5
                pc = if f.z { 0x1400cabc5 } else { 0x1400ca9b3 };
                continue;
            }
            0x1400ca9b3 => {
                // 1400ca9b3  4d858412f0e80e00  TEST qword ptr [R10 + RDX*0x1 + 0xee8f0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xee8f0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca9bb  741c  JZ 0x1400ca9d9
                pc = if f.z { 0x1400ca9d9 } else { 0x1400ca9bd };
                continue;
            }
            0x1400ca9bd => {
                // 1400ca9bd  488bc7  MOV RAX,RDI
                {
                    r[0] = r[7];
                }
                // 1400ca9c0  81c132001400  ADD ECX,0x140032
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x140032u64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400ca9c6  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400ca9c9  4c8584c2d02a1100  TEST qword ptr [RDX + RAX*0x8 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca9d1  7506  JNZ 0x1400ca9d9
                pc = if !f.z { 0x1400ca9d9 } else { 0x1400ca9d3 };
                continue;
            }
            0x1400ca9d3 => {
                // 1400ca9d3  81c120001800  ADD ECX,0x180020
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x180020u64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca9d9;
            }
            0x1400ca9d9 => {
                // 1400ca9d9  83e707  AND EDI,0x7
                {
                    let v = (r[7] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 1400ca9dc  4c8584fad02a1100  TEST qword ptr [RDX + RDI*0x8 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400ca9e4  7512  JNZ 0x1400ca9f8
                pc = if !f.z { 0x1400ca9f8 } else { 0x1400ca9e6 };
                continue;
            }
            0x1400ca9e6 => {
                // 1400ca9e6  81c138002a00  ADD ECX,0x2a0038
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x2a0038u64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400ca9ec;
            }
            0x1400ca9ec => {
                // 1400ca9ec  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400ca9f3  e9aa000000  JMP 0x1400caaa2
                pc = 0x1400caaa2;
                continue;
            }
            0x1400ca9f8 => {
                // 1400ca9f8  4d85841280360f00  TEST qword ptr [R10 + RDX*0x1 + 0xf3680],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3680u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400caa00  75ea  JNZ 0x1400ca9ec
                pc = if !f.z { 0x1400ca9ec } else { 0x1400caa02 };
                continue;
            }
            0x1400caa02 => {
                // 1400caa02  4985b412803a0f00  TEST qword ptr [R10 + RDX*0x1 + 0xf3a80],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3a80u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400caa0a  bf08000000  MOV EDI,0x8
                {
                    r[7] = (0x8u64) & 0xffff_ffff;
                }
                // 1400caa0f  410f95c1  SETNZ R9B
                {
                    r[9] = (r[9] & !0xffu64) | ((u64::from(!f.z)) & 0xffu64);
                }
                // 1400caa13  4d85841280460f00  TEST qword ptr [R10 + RDX*0x1 + 0xf4680],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf4680u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400caa1b  7559  JNZ 0x1400caa76
                pc = if !f.z { 0x1400caa76 } else { 0x1400caa1d };
                continue;
            }
            0x1400caa1d => {
                // 1400caa1d  8d43f8  LEA EAX,[RBX + -0x8]
                {
                    r[0] =
                        (0u64.wrapping_add(r[3]).wrapping_add(0xfffffffffffffff8u64)) & 0xffff_ffff;
                }
                // 1400caa20  bf10000000  MOV EDI,0x10
                {
                    r[7] = (0x10u64) & 0xffff_ffff;
                }
                // 1400caa25  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400caa28  4c8d35d155f3ff  LEA R14,[0x140000000]
                {
                    r[14] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400caa2f  4985b4d6803a0f00  TEST qword ptr [R14 + RDX*0x8 + 0xf3a80],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3a80u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400caa37  0f95c0  SETNZ AL
                {
                    r[0] = (r[0] & !0xffu64) | ((u64::from(!f.z)) & 0xffu64);
                }
                // 1400caa3a  410ac1  OR AL,R9B
                {
                    let v = (r[0] & 0xffu64 | r[9] & 0xffu64) & 0xffu64;
                    f.logic(v, 8);
                    r[0] = (r[0] & !0xffu64) | ((v) & 0xffu64);
                }
                // 1400caa3d  440fb6c8  MOVZX R9D,AL
                {
                    r[9] = (r[0] & 0xffu64) & 0xffff_ffff;
                }
                // 1400caa41  440fb6d0  MOVZX R10D,AL
                {
                    r[10] = (r[0] & 0xffu64) & 0xffff_ffff;
                }
                // 1400caa45  4d8584d680460f00  TEST qword ptr [R14 + RDX*0x8 + 0xf4680],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4680u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400caa4d  7520  JNZ 0x1400caa6f
                pc = if !f.z { 0x1400caa6f } else { 0x1400caa4f };
                continue;
            }
            0x1400caa4f => {
                // 1400caa4f  8d43f0  LEA EAX,[RBX + -0x10]
                {
                    r[0] =
                        (0u64.wrapping_add(r[3]).wrapping_add(0xfffffffffffffff0u64)) & 0xffff_ffff;
                }
                // 1400caa52  450fb6ca  MOVZX R9D,R10B
                {
                    r[9] = (r[10] & 0xffu64) & 0xffff_ffff;
                }
                // 1400caa56  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400caa59  bf18000000  MOV EDI,0x18
                {
                    r[7] = (0x18u64) & 0xffff_ffff;
                }
                // 1400caa5e  b801000000  MOV EAX,0x1
                {
                    r[0] = (0x1u64) & 0xffff_ffff;
                }
                // 1400caa63  4985b4d6803a0f00  TEST qword ptr [R14 + RDX*0x8 + 0xf3a80],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[2].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3a80u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400caa6b  440f45c8  CMOVNZ R9D,EAX
                {
                    if !f.z {
                        r[9] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                pc = 0x1400caa6f;
            }
            0x1400caa6f => {
                // 1400caa6f  488d158a55f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                pc = 0x1400caa76;
            }
            0x1400caa76 => {
                // 1400caa76  4584c9  TEST R9B,R9B
                {
                    let v = (r[9] & 0xffu64) & 0xffu64;
                    f.logic(v, 8);
                }
                // 1400caa79  7513  JNZ 0x1400caa8e
                pc = if !f.z { 0x1400caa8e } else { 0x1400caa7b };
                continue;
            }
            0x1400caa7b => {
                // 1400caa7b  2bdf  SUB EBX,EDI
                {
                    let v = f.arithmetic(r[3] & 0xffffffffu64, r[7] & 0xffffffffu64, 32, true);
                    r[3] = (v) & 0xffff_ffff;
                }
                // 1400caa7d  4863c3  MOVSXD RAX,EBX
                {
                    r[0] = ((r[3] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400caa80  4885b4c280460f00  TEST qword ptr [RDX + RAX*0x8 + 0xf4680],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4680u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400caa88  0f84c4030000  JZ 0x1400cae52
                pc = if f.z { 0x1400cae52 } else { 0x1400caa8e };
                continue;
            }
            0x1400caa8e => {
                // 1400caa8e  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400caa95  81c134002700  ADD ECX,0x270034
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x270034u64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400caa9b;
            }
            0x1400caa9b => {
                // 1400caa9b  448b8d80000000  MOV R9D,dword ptr [RBP + 0x80]
                {
                    r[9] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x80u64))) & 0xffff_ffff;
                }
                pc = 0x1400caaa2;
            }
            0x1400caaa2 => {
                // 1400caaa2  4c8d055755f3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                pc = 0x1400caaa9;
            }
            0x1400caaa9 => {
                // 1400caaa9  498d45ff  LEA RAX,[R13 + -0x1]
                {
                    r[0] = 0u64.wrapping_add(r[13]).wrapping_add(0xffffffffffffffffu64);
                }
                // 1400caaad  4c23e8  AND R13,RAX
                {
                    let v = r[13] & r[0];
                    f.logic(v, 64);
                    r[13] = v;
                }
                // 1400caab0  0f854afdffff  JNZ 0x1400ca800
                pc = if !f.z { 0x1400ca800 } else { 0x1400caab6 };
                continue;
            }
            0x1400caab6 => {
                // 1400caab6  44898d80000000  MOV dword ptr [RBP + 0x80],R9D
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[5]).wrapping_add(0x80u64),
                        r[9] & 0xffffffffu64,
                    );
                }
                // 1400caabd  4533e4  XOR R12D,R12D
                {
                    let v = 0u64;
                    f.logic(v, 32);
                    r[12] = (v) & 0xffff_ffff;
                }
                pc = 0x1400caac0;
            }
            0x1400caac0 => {
                // 1400caac0  4d0933  OR qword ptr [R11],R14
                {
                    let v = m.read::<64>(0u64.wrapping_add(r[11])) | r[14];
                    f.logic(v, 64);
                    m.write::<64>(0u64.wrapping_add(r[11]), v);
                }
                // 1400caac3  49bd0000000000000800  MOV R13,0x8000000000000
                {
                    r[13] = 0x8000000000000u64;
                }
                // 1400caacd  488b9598000000  MOV RDX,qword ptr [RBP + 0x98]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x98u64));
                }
                // 1400caad4  48231424  AND RDX,qword ptr [RSP]
                {
                    let v = r[2] & m.read::<64>(0u64.wrapping_add(r[4]));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400caad8  480b1531662f00  OR RDX,qword ptr [0x1403c1110]
                {
                    let v = r[2] | m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400caadf  498b4328  MOV RAX,qword ptr [R11 + 0x28]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x28u64));
                }
                // 1400caae3  8bbd88000000  MOV EDI,dword ptr [RBP + 0x88]
                {
                    r[7] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x88u64))) & 0xffff_ffff;
                }
                // 1400caae9  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400caaec  4823d0  AND RDX,RAX
                {
                    let v = r[2] & r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400caaef  498b4320  MOV RAX,qword ptr [R11 + 0x20]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x20u64));
                }
                // 1400caaf3  4823542408  AND RDX,qword ptr [RSP + 0x8]
                {
                    let v = r[2] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x8u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400caaf8  48f7d0  NOT RAX
                {
                    let v = !(r[0]);
                    r[0] = v;
                }
                // 1400caafb  49895308  MOV qword ptr [R11 + 0x8],RDX
                {
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x8u64), r[2]);
                }
                // 1400caaff  488b542410  MOV RDX,qword ptr [RSP + 0x10]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x10u64));
                }
                // 1400cab04  4823542418  AND RDX,qword ptr [RSP + 0x18]
                {
                    let v = r[2] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x18u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cab09  480b1508662f00  OR RDX,qword ptr [0x1403c1118]
                {
                    let v = r[2] | m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cab10  4823d0  AND RDX,RAX
                {
                    let v = r[2] & r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cab13  4823542420  AND RDX,qword ptr [RSP + 0x20]
                {
                    let v = r[2] & m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x20u64));
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cab18  49895310  MOV qword ptr [R11 + 0x10],RDX
                {
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x10u64), r[2]);
                }
                // 1400cab1c  4c8b0d75662f00  MOV R9,qword ptr [0x1403c1198]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(0x1403c1198u64));
                }
                // 1400cab23  4c8b3d0e662f00  MOV R15,qword ptr [0x1403c1138]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(0x1403c1138u64));
                }
                // 1400cab2a  498bd9  MOV RBX,R9
                {
                    r[3] = r[9];
                }
                // 1400cab2d  4c8b35fc652f00  MOV R14,qword ptr [0x1403c1130]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(0x1403c1130u64));
                }
                // 1400cab34  4d8bc1  MOV R8,R9
                {
                    r[8] = r[9];
                }
                // 1400cab37  f3490fb8c7  POPCNT RAX,R15
                {
                    let v = r[15];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 1400cab3c  4d8bd1  MOV R10,R9
                {
                    r[10] = r[9];
                }
                // 1400cab3f  48c1eb2a  SHR RBX,0x2a
                {
                    let v = r[3];
                    let n = 42u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[3] = v;
                    }
                }
                // 1400cab43  f3490fb8d6  POPCNT RDX,R14
                {
                    let v = r[14];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[2] = u64::from(v.count_ones());
                }
                // 1400cab48  2bd0  SUB EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cab4a  49c1e92f  SHR R9,0x2f
                {
                    let v = r[9];
                    let n = 47u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cab4e  83e31f  AND EBX,0x1f
                {
                    let v = (r[3] & 0xffffffffu64 & 0x1fu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[3] = (v) & 0xffff_ffff;
                }
                // 1400cab51  49c1ea25  SHR R10,0x25
                {
                    let v = r[10];
                    let n = 37u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[10] = v;
                    }
                }
                // 1400cab55  4183e11f  AND R9D,0x1f
                {
                    let v = (r[9] & 0xffffffffu64 & 0x1fu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400cab59  49c1e820  SHR R8,0x20
                {
                    let v = r[8];
                    let n = 32u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cab5d  4183e21f  AND R10D,0x1f
                {
                    let v = (r[10] & 0xffffffffu64 & 0x1fu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[10] = (v) & 0xffff_ffff;
                }
                // 1400cab61  4183e01f  AND R8D,0x1f
                {
                    let v = (r[8] & 0xffffffffu64 & 0x1fu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 1400cab65  418d0419  LEA EAX,[R9 + RBX*0x1]
                {
                    r[0] = (0u64
                        .wrapping_add(r[9])
                        .wrapping_add(r[3].wrapping_mul(0x1u64)))
                        & 0xffff_ffff;
                }
                // 1400cab69  4103c2  ADD EAX,R10D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[10] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cab6c  4103c0  ADD EAX,R8D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cab6f  0fafd0  IMUL EDX,EAX
                {
                    r[2] =
                        ((r[2] & 0xffffffffu64).wrapping_mul(r[0] & 0xffffffffu64)) & 0xffff_ffff;
                }
                // 1400cab72  8b8580000000  MOV EAX,dword ptr [RBP + 0x80]
                {
                    r[0] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x80u64))) & 0xffff_ffff;
                }
                // 1400cab78  2bc7  SUB EAX,EDI
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[7] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cab7a  488b3d37662f00  MOV RDI,qword ptr [0x1403c11b8]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(0x1403c11b8u64));
                }
                // 1400cab81  488bf7  MOV RSI,RDI
                {
                    r[6] = r[7];
                }
                // 1400cab84  69d202000200  IMUL EDX,EDX,0x20002
                {
                    r[2] = ((r[2] & 0xffffffffu64).wrapping_mul(0x20002u64)) & 0xffff_ffff;
                }
                // 1400cab8a  8d0480  LEA EAX,[RAX + RAX*0x4]
                {
                    r[0] = (0u64
                        .wrapping_add(r[0])
                        .wrapping_add(r[0].wrapping_mul(0x4u64)))
                        & 0xffff_ffff;
                }
                // 1400cab8d  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cab8f  48baaa55aa55aa55aa55  MOV RDX,0x55aa55aa55aa55aa
                {
                    r[2] = 0x55aa55aa55aa55aau64;
                }
                // 1400cab99  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cab9b  48b80000000000000200  MOV RAX,0x2000000000000
                {
                    r[0] = 0x2000000000000u64;
                }
                // 1400caba5  4823f0  AND RSI,RAX
                {
                    let v = r[6] & r[0];
                    f.logic(v, 64);
                    r[6] = v;
                }
                // 1400caba8  0f84be020000  JZ 0x1400cae6c
                pc = if f.z { 0x1400cae6c } else { 0x1400cabae };
                continue;
            }
            0x1400cabae => {
                // 1400cabae  8bc3  MOV EAX,EBX
                {
                    r[0] = (r[3] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cabb0  412bc1  SUB EAX,R9D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[9] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cabb3  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400cabb4  33c2  XOR EAX,EDX
                {
                    let v = (r[0] & 0xffffffffu64 ^ r[2] & 0xffffffffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cabb6  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cabb8  69c0fefffdff  IMUL EAX,EAX,-0x20002
                {
                    r[0] =
                        ((r[0] & 0xffffffffu64).wrapping_mul(0xfffffffffffdfffeu64)) & 0xffff_ffff;
                }
                // 1400cabbe  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cabc0  e9f1020000  JMP 0x1400caeb6
                pc = 0x1400caeb6;
                continue;
            }
            0x1400cabc5 => {
                // 1400cabc5  4d858412f0e80e00  TEST qword ptr [R10 + RDX*0x1 + 0xee8f0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xee8f0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400cabcd  741c  JZ 0x1400cabeb
                pc = if f.z { 0x1400cabeb } else { 0x1400cabcf };
                continue;
            }
            0x1400cabcf => {
                // 1400cabcf  488bc7  MOV RAX,RDI
                {
                    r[0] = r[7];
                }
                // 1400cabd2  81c132001400  ADD ECX,0x140032
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x140032u64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cabd8  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cabdb  4c8584c2d02a1100  TEST qword ptr [RDX + RAX*0x8 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400cabe3  7506  JNZ 0x1400cabeb
                pc = if !f.z { 0x1400cabeb } else { 0x1400cabe5 };
                continue;
            }
            0x1400cabe5 => {
                // 1400cabe5  81c16e005200  ADD ECX,0x52006e
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x52006eu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400cabeb;
            }
            0x1400cabeb => {
                // 1400cabeb  83e707  AND EDI,0x7
                {
                    let v = (r[7] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 1400cabee  4c8584fad02a1100  TEST qword ptr [RDX + RDI*0x8 + 0x112ad0],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0x112ad0u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400cabf6  7521  JNZ 0x1400cac19
                pc = if !f.z { 0x1400cac19 } else { 0x1400cabf8 };
                continue;
            }
            0x1400cabf8 => {
                // 1400cabf8  b83f000000  MOV EAX,0x3f
                {
                    r[0] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400cabfd  2bc3  SUB EAX,EBX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[3] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cabff  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cac02  488d05f753f3ff  LEA RAX,[0x140000000]
                {
                    r[0] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cac09  038c90e00f1000  ADD ECX,dword ptr [RAX + RDX*0x4 + 0x100fe0]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[0])
                                .wrapping_add(r[2].wrapping_mul(0x4u64))
                                .wrapping_add(0x100fe0u64),
                        ),
                        32,
                        false,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cac10  488d15e953f3ff  LEA RDX,[0x140000000]
                {
                    r[2] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cac17  eb10  JMP 0x1400cac29
                pc = 0x1400cac29;
                continue;
            }
            0x1400cac19 => {
                // 1400cac19  4d85841280360f00  TEST qword ptr [R10 + RDX*0x1 + 0xf3680],R8
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3680u64),
                    ) & r[8];
                    f.logic(v, 64);
                }
                // 1400cac21  7506  JNZ 0x1400cac29
                pc = if !f.z { 0x1400cac29 } else { 0x1400cac23 };
                continue;
            }
            0x1400cac23 => {
                // 1400cac23  81c178005a00  ADD ECX,0x5a0078
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, 0x5a0078u64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400cac29;
            }
            0x1400cac29 => {
                // 1400cac29  4d8b8c1280320f00  MOV R9,qword ptr [R10 + RDX*0x1 + 0xf3280]
                {
                    r[9] = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3280u64),
                    );
                }
                // 1400cac31  4c23ce  AND R9,RSI
                {
                    let v = r[9] & r[6];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cac34  0f8563010000  JNZ 0x1400cad9d
                pc = if !f.z { 0x1400cad9d } else { 0x1400cac3a };
                continue;
            }
            0x1400cac3a => {
                // 1400cac3a  b83f000000  MOV EAX,0x3f
                {
                    r[0] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400cac3f  2bc3  SUB EAX,EBX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[3] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cac41  4c63c0  MOVSXD R8,EAX
                {
                    r[8] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cac44  49c1e007  SHL R8,0x7
                {
                    let v = r[8];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cac48  412b8c10202b1000  SUB ECX,dword ptr [R8 + RDX*0x1 + 0x102b20]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[8])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0x102b20u64),
                        ),
                        32,
                        true,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cac50  4885b4fa902a1100  TEST qword ptr [RDX + RDI*0x8 + 0x112a90],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0x112a90u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400cac58  740a  JZ 0x1400cac64
                pc = if f.z { 0x1400cac64 } else { 0x1400cac5a };
                continue;
            }
            0x1400cac5a => {
                // 1400cac5a  4885b4fa502a1100  TEST qword ptr [RDX + RDI*0x8 + 0x112a50],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0x112a50u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400cac62  7508  JNZ 0x1400cac6c
                pc = if !f.z { 0x1400cac6c } else { 0x1400cac64 };
                continue;
            }
            0x1400cac64 => {
                // 1400cac64  412b8c10242b1000  SUB ECX,dword ptr [R8 + RDX*0x1 + 0x102b24]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[8])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0x102b24u64),
                        ),
                        32,
                        true,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400cac6c;
            }
            0x1400cac6c => {
                // 1400cac6c  4d85f6  TEST R14,R14
                {
                    let v = r[14];
                    f.logic(v, 64);
                }
                // 1400cac6f  7408  JZ 0x1400cac79
                pc = if f.z { 0x1400cac79 } else { 0x1400cac71 };
                continue;
            }
            0x1400cac71 => {
                // 1400cac71  412b8c10282b1000  SUB ECX,dword ptr [R8 + RDX*0x1 + 0x102b28]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[8])
                                .wrapping_add(r[2].wrapping_mul(0x1u64))
                                .wrapping_add(0x102b28u64),
                        ),
                        32,
                        true,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400cac79;
            }
            0x1400cac79 => {
                // 1400cac79  4d8b9412f0ea0e00  MOV R10,qword ptr [R10 + RDX*0x1 + 0xeeaf0]
                {
                    r[10] = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xeeaf0u64),
                    );
                }
                // 1400cac81  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400cac88  4d8bca  MOV R9,R10
                {
                    r[9] = r[10];
                }
                // 1400cac8b  4d23ce  AND R9,R14
                {
                    let v = r[9] & r[14];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cac8e  746e  JZ 0x1400cacfe
                pc = if f.z { 0x1400cacfe } else { 0x1400cac90 };
                continue;
            }
            0x1400cac90 => {
                // 1400cac90  490fbcd1  BSF RDX,R9
                {
                    let v = r[9];
                    f.z = v == 0;
                    if v != 0 {
                        r[2] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400cac94  488d056553f3ff  LEA RAX,[0x140000000]
                {
                    r[0] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cac9b  458b84002c2b1000  MOV R8D,dword ptr [R8 + RAX*0x1 + 0x102b2c]
                {
                    r[8] = (m.read::<32>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[0].wrapping_mul(0x1u64))
                            .wrapping_add(0x102b2cu64),
                    )) & 0xffff_ffff;
                }
                // 1400caca3  b83f000000  MOV EAX,0x3f
                {
                    r[0] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400caca8  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cacaa  891568462300  MOV dword ptr [0x1402ff318],EDX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[2] & 0xffffffffu64);
                }
                // 1400cacb0  8bd0  MOV EDX,EAX
                {
                    r[2] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cacb2  488d054753f3ff  LEA RAX,[0x140000000]
                {
                    r[0] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cacb9  48c1e207  SHL RDX,0x7
                {
                    let v = r[2];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cacbd  8b84022c2b1000  MOV EAX,dword ptr [RDX + RAX*0x1 + 0x102b2c]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x1u64))
                            .wrapping_add(0x102b2cu64),
                    )) & 0xffff_ffff;
                }
                // 1400cacc4  4103c0  ADD EAX,R8D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cacc7  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cacc9  49ffc9  DEC R9
                {
                    let v = f.arithmetic(r[9], 1u64, 64, true);
                    r[9] = v;
                }
                // 1400caccc  4d23ca  AND R9,R10
                {
                    let v = r[9] & r[10];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400caccf  4d23ce  AND R9,R14
                {
                    let v = r[9] & r[14];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cacd2  742a  JZ 0x1400cacfe
                pc = if f.z { 0x1400cacfe } else { 0x1400cacd4 };
                continue;
            }
            0x1400cacd4 => {
                // 1400cacd4  490fbcd1  BSF RDX,R9
                {
                    let v = r[9];
                    f.z = v == 0;
                    if v != 0 {
                        r[2] = u64::from(v.trailing_zeros());
                    }
                }
                // 1400cacd8  b83f000000  MOV EAX,0x3f
                {
                    r[0] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400cacdd  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cacdf  891533462300  MOV dword ptr [0x1402ff318],EDX
                {
                    m.write::<32>(0u64.wrapping_add(0x1402ff318u64), r[2] & 0xffffffffu64);
                }
                // 1400cace5  8bd0  MOV EDX,EAX
                {
                    r[2] = (r[0] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cace7  488d051253f3ff  LEA RAX,[0x140000000]
                {
                    r[0] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cacee  48c1e207  SHL RDX,0x7
                {
                    let v = r[2];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cacf2  8b84022c2b1000  MOV EAX,dword ptr [RDX + RAX*0x1 + 0x102b2c]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[2])
                            .wrapping_add(r[0].wrapping_mul(0x1u64))
                            .wrapping_add(0x102b2cu64),
                    )) & 0xffff_ffff;
                }
                // 1400cacf9  4103c0  ADD EAX,R8D
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[8] & 0xffffffffu64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cacfc  2bc8  SUB ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400cacfe;
            }
            0x1400cacfe => {
                // 1400cacfe  498bc5  MOV RAX,R13
                {
                    r[0] = r[13];
                }
                // 1400cad01  48f7d8  NEG RAX
                {
                    let v = f.arithmetic(0u64, r[0], 64, true);
                    r[0] = v;
                }
                // 1400cad04  4923c5  AND RAX,R13
                {
                    let v = r[0] & r[13];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cad07  4c0bf0  OR R14,RAX
                {
                    let v = r[14] | r[0];
                    f.logic(v, 64);
                    r[14] = v;
                }
                // 1400cad0a  4c89b590000000  MOV qword ptr [RBP + 0x90],R14
                {
                    m.write::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64), r[14]);
                }
                // 1400cad11  83fb28  CMP EBX,0x28
                {
                    f.arithmetic(r[3] & 0xffffffffu64, 0x28u64, 32, true);
                }
                // 1400cad14  0f8d81fdffff  JGE 0x1400caa9b
                pc = if f.s == f.o { 0x1400caa9b } else { 0x1400cad1a };
                continue;
            }
            0x1400cad1a => {
                // 1400cad1a  ba27000000  MOV EDX,0x27
                {
                    r[2] = (0x27u64) & 0xffff_ffff;
                }
                // 1400cad1f  b827000000  MOV EAX,0x27
                {
                    r[0] = (0x27u64) & 0xffff_ffff;
                }
                // 1400cad24  3bda  CMP EBX,EDX
                {
                    f.arithmetic(r[3] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                }
                // 1400cad26  0f4cd3  CMOVL EDX,EBX
                {
                    if f.s != f.o {
                        r[2] = (r[3] & 0xffffffffu64) & 0xffff_ffff;
                    }
                }
                // 1400cad29  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cad2b  4c63d0  MOVSXD R10,EAX
                {
                    r[10] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cad2e  8d43f8  LEA EAX,[RBX + -0x8]
                {
                    r[0] =
                        (0u64.wrapping_add(r[3]).wrapping_add(0xfffffffffffffff8u64)) & 0xffff_ffff;
                }
                // 1400cad31  4c63c0  MOVSXD R8,EAX
                {
                    r[8] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cad34  488d1dc552f3ff  LEA RBX,[0x140000000]
                {
                    r[3] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cad3b  49c1e206  SHL R10,0x6
                {
                    let v = r[10];
                    let n = 6u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[10] = v;
                    }
                }
                // 1400cad3f  4b8d0404  LEA RAX,[R12 + R8*0x1]
                {
                    r[0] = 0u64
                        .wrapping_add(r[12])
                        .wrapping_add(r[8].wrapping_mul(0x1u64));
                }
                // 1400cad43  0fbe9418707d0f00  MOVSX EDX,byte ptr [RAX + RBX*0x1 + 0xf7d70]
                {
                    r[2] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[3].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400cad4b  b867666666  MOV EAX,0x66666667
                {
                    r[0] = (0x66666667u64) & 0xffff_ffff;
                }
                // 1400cad50  410faf941a1c4b1000  IMUL EDX,dword ptr [R10 + RBX*0x1 + 0x104b1c]
                {
                    r[2] = ((r[2] & 0xffffffffu64).wrapping_mul(
                        m.read::<32>(
                            0u64.wrapping_add(r[10])
                                .wrapping_add(r[3].wrapping_mul(0x1u64))
                                .wrapping_add(0x104b1cu64),
                        ),
                    )) & 0xffff_ffff;
                }
                // 1400cad59  f7ea  IMUL EDX
                {
                    let v = (r[0] as i32 as i128) * ((r[2] & 0xffffffffu64) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 1400cad5b  448bca  MOV R9D,EDX
                {
                    r[9] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cad5e  41c1f902  SAR R9D,0x2
                {
                    let v = r[9] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[9] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cad62  418bc1  MOV EAX,R9D
                {
                    r[0] = (r[9] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cad65  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cad68  4403c8  ADD R9D,EAX
                {
                    let v = f.arithmetic(r[9] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400cad6b  4b8d0407  LEA RAX,[R15 + R8*0x1]
                {
                    r[0] = 0u64
                        .wrapping_add(r[15])
                        .wrapping_add(r[8].wrapping_mul(0x1u64));
                }
                // 1400cad6f  440fbe8418707d0f00  MOVSX R8D,byte ptr [RAX + RBX*0x1 + 0xf7d70]
                {
                    r[8] = (((m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[3].wrapping_mul(0x1u64))
                            .wrapping_add(0xf7d70u64),
                    )) as i8 as i64) as u64)
                        & 0xffff_ffff;
                }
                // 1400cad78  b867666666  MOV EAX,0x66666667
                {
                    r[0] = (0x66666667u64) & 0xffff_ffff;
                }
                // 1400cad7d  450faf841a184b1000  IMUL R8D,dword ptr [R10 + RBX*0x1 + 0x104b18]
                {
                    r[8] = ((r[8] & 0xffffffffu64).wrapping_mul(
                        m.read::<32>(
                            0u64.wrapping_add(r[10])
                                .wrapping_add(r[3].wrapping_mul(0x1u64))
                                .wrapping_add(0x104b18u64),
                        ),
                    )) & 0xffff_ffff;
                }
                // 1400cad86  41f7e8  IMUL R8D
                {
                    let v = (r[0] as i32 as i128) * ((r[8] & 0xffffffffu64) as i32 as i128);
                    r[0] = v as u64 & 0xffffffffu64;
                    r[2] = (v >> 32) as u64 & 0xffffffffu64;
                }
                // 1400cad89  c1fa02  SAR EDX,0x2
                {
                    let v = r[2] & 0xffffffffu64;
                    let n = 2u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[2] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cad8c  8bc2  MOV EAX,EDX
                {
                    r[0] = (r[2] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cad8e  c1e81f  SHR EAX,0x1f
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 31u32;
                    if n != 0 {
                        let v = (v >> n) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cad91  03d0  ADD EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cad93  412bd1  SUB EDX,R9D
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[9] & 0xffffffffu64, 32, true);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cad96  03ca  ADD ECX,EDX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cad98  e9fefcffff  JMP 0x1400caa9b
                pc = 0x1400caa9b;
                continue;
            }
            0x1400cad9d => {
                // 1400cad9d  498b841280360f00  MOV RAX,qword ptr [R10 + RDX*0x1 + 0xf3680]
                {
                    r[0] = m.read::<64>(
                        0u64.wrapping_add(r[10])
                            .wrapping_add(r[2].wrapping_mul(0x1u64))
                            .wrapping_add(0xf3680u64),
                    );
                }
                // 1400cada5  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cada8  f3480fb8d0  POPCNT RDX,RAX
                {
                    let v = r[0];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[2] = u64::from(v.count_ones());
                }
                // 1400cadad  f3490fb8c1  POPCNT RAX,R9
                {
                    let v = r[9];
                    f.z = v == 0;
                    f.o = false;
                    f.s = false;
                    r[0] = u64::from(v.count_ones());
                }
                // 1400cadb2  3bc2  CMP EAX,EDX
                {
                    f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                }
                // 1400cadb4  b83f000000  MOV EAX,0x3f
                {
                    r[0] = (0x3fu64) & 0xffff_ffff;
                }
                // 1400cadb9  7f6c  JG 0x1400cae27
                pc = if !f.z && f.s == f.o {
                    0x1400cae27
                } else {
                    0x1400cadbb
                };
                continue;
            }
            0x1400cadbb => {
                // 1400cadbb  2bc3  SUB EAX,EBX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[3] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cadbd  4c8d0d3c52f3ff  LEA R9,[0x140000000]
                {
                    r[9] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cadc4  4c63c0  MOVSXD R8,EAX
                {
                    r[8] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cadc7  49c1e007  SHL R8,0x7
                {
                    let v = r[8];
                    let n = 7u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cadcb  438b8408202b1000  MOV EAX,dword ptr [R8 + R9*0x1 + 0x102b20]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[9].wrapping_mul(0x1u64))
                            .wrapping_add(0x102b20u64),
                    )) & 0xffff_ffff;
                }
                // 1400cadd3  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400cadd4  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cadd6  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cadd8  f7d8  NEG EAX
                {
                    let v = f.arithmetic(0u64, r[0] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cadda  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400caddc  4985b4f9902a1100  TEST qword ptr [R9 + RDI*0x8 + 0x112a90],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0x112a90u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400cade4  740a  JZ 0x1400cadf0
                pc = if f.z { 0x1400cadf0 } else { 0x1400cade6 };
                continue;
            }
            0x1400cade6 => {
                // 1400cade6  4985b4f9502a1100  TEST qword ptr [R9 + RDI*0x8 + 0x112a50],RSI
                {
                    let v = m.read::<64>(
                        0u64.wrapping_add(r[9])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0x112a50u64),
                    ) & r[6];
                    f.logic(v, 64);
                }
                // 1400cadee  7511  JNZ 0x1400cae01
                pc = if !f.z { 0x1400cae01 } else { 0x1400cadf0 };
                continue;
            }
            0x1400cadf0 => {
                // 1400cadf0  438b8408242b1000  MOV EAX,dword ptr [R8 + R9*0x1 + 0x102b24]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[9].wrapping_mul(0x1u64))
                            .wrapping_add(0x102b24u64),
                    )) & 0xffff_ffff;
                }
                // 1400cadf8  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400cadf9  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cadfb  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cadfd  f7d8  NEG EAX
                {
                    let v = f.arithmetic(0u64, r[0] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cadff  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400cae01;
            }
            0x1400cae01 => {
                // 1400cae01  4d85f6  TEST R14,R14
                {
                    let v = r[14];
                    f.logic(v, 64);
                }
                // 1400cae04  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400cae0b  0f848afcffff  JZ 0x1400caa9b
                pc = if f.z { 0x1400caa9b } else { 0x1400cae11 };
                continue;
            }
            0x1400cae11 => {
                // 1400cae11  438b8408282b1000  MOV EAX,dword ptr [R8 + R9*0x1 + 0x102b28]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[8])
                            .wrapping_add(r[9].wrapping_mul(0x1u64))
                            .wrapping_add(0x102b28u64),
                    )) & 0xffff_ffff;
                }
                // 1400cae19  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400cae1a  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cae1c  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cae1e  f7d8  NEG EAX
                {
                    let v = f.arithmetic(0u64, r[0] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cae20  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cae22  e974fcffff  JMP 0x1400caa9b
                pc = 0x1400caa9b;
                continue;
            }
            0x1400cae27 => {
                // 1400cae27  448b8d80000000  MOV R9D,dword ptr [RBP + 0x80]
                {
                    r[9] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x80u64))) & 0xffff_ffff;
                }
                // 1400cae2e  4c8d05cb51f3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cae35  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400cae3c  2bc3  SUB EAX,EBX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[3] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cae3e  4863d0  MOVSXD RDX,EAX
                {
                    r[2] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cae41  48c1e205  SHL RDX,0x5
                {
                    let v = r[2];
                    let n = 5u32;
                    if n != 0 {
                        let v = v.wrapping_shl(n);
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cae45  422b8c02e8221000  SUB ECX,dword ptr [RDX + R8*0x1 + 0x1022e8]
                {
                    let v = f.arithmetic(
                        r[1] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[2])
                                .wrapping_add(r[8].wrapping_mul(0x1u64))
                                .wrapping_add(0x1022e8u64),
                        ),
                        32,
                        true,
                    );
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cae4d  e957fcffff  JMP 0x1400caaa9
                pc = 0x1400caaa9;
                continue;
            }
            0x1400cae52 => {
                // 1400cae52  448b8d80000000  MOV R9D,dword ptr [RBP + 0x80]
                {
                    r[9] =
                        (m.read::<32>(0u64.wrapping_add(r[5]).wrapping_add(0x80u64))) & 0xffff_ffff;
                }
                // 1400cae59  4c8d05a051f3ff  LEA R8,[0x140000000]
                {
                    r[8] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400cae60  4c8bb590000000  MOV R14,qword ptr [RBP + 0x90]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[5]).wrapping_add(0x90u64));
                }
                // 1400cae67  e93dfcffff  JMP 0x1400caaa9
                pc = 0x1400caaa9;
                continue;
            }
            0x1400cae6c => {
                // 1400cae6c  4c85f2  TEST RDX,R14
                {
                    let v = r[2] & r[14];
                    f.logic(v, 64);
                }
                // 1400cae6f  741c  JZ 0x1400cae8d
                pc = if f.z { 0x1400cae8d } else { 0x1400cae71 };
                continue;
            }
            0x1400cae71 => {
                // 1400cae71  418bd4  MOV EDX,R12D
                {
                    r[2] = (r[12] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cae74  4985fd  TEST R13,RDI
                {
                    let v = r[13] & r[7];
                    f.logic(v, 64);
                }
                // 1400cae77  7507  JNZ 0x1400cae80
                pc = if !f.z { 0x1400cae80 } else { 0x1400cae79 };
                continue;
            }
            0x1400cae79 => {
                // 1400cae79  4169d008000800  IMUL EDX,R8D,0x80008
                {
                    r[2] = ((r[8] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                pc = 0x1400cae80;
            }
            0x1400cae80 => {
                // 1400cae80  4169c208000800  IMUL EAX,R10D,0x80008
                {
                    r[0] = ((r[10] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                // 1400cae87  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cae89  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400cae8b  eb29  JMP 0x1400caeb6
                pc = 0x1400caeb6;
                continue;
            }
            0x1400cae8d => {
                // 1400cae8d  48b855aa55aa55aa55aa  MOV RAX,-0x55aa55aa55aa55ab
                {
                    r[0] = 0xaa55aa55aa55aa55u64;
                }
                // 1400cae97  4c85f0  TEST RAX,R14
                {
                    let v = r[0] & r[14];
                    f.logic(v, 64);
                }
                // 1400cae9a  742e  JZ 0x1400caeca
                pc = if f.z { 0x1400caeca } else { 0x1400cae9c };
                continue;
            }
            0x1400cae9c => {
                // 1400cae9c  418bc4  MOV EAX,R12D
                {
                    r[0] = (r[12] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400cae9f  4985fd  TEST R13,RDI
                {
                    let v = r[13] & r[7];
                    f.logic(v, 64);
                }
                // 1400caea2  7507  JNZ 0x1400caeab
                pc = if !f.z { 0x1400caeab } else { 0x1400caea4 };
                continue;
            }
            0x1400caea4 => {
                // 1400caea4  4169c208000800  IMUL EAX,R10D,0x80008
                {
                    r[0] = ((r[10] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                pc = 0x1400caeab;
            }
            0x1400caeab => {
                // 1400caeab  4169d008000800  IMUL EDX,R8D,0x80008
                {
                    r[2] = ((r[8] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                // 1400caeb2  2bd0  SUB EDX,EAX
                {
                    let v = f.arithmetic(r[2] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400caeb4  03ca  ADD ECX,EDX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400caeb6;
            }
            0x1400caeb6 => {
                // 1400caeb6  48baaa55aa55aa55aa55  MOV RDX,0x55aa55aa55aa55aa
                {
                    r[2] = 0x55aa55aa55aa55aau64;
                }
                // 1400caec0  48b855aa55aa55aa55aa  MOV RAX,-0x55aa55aa55aa55ab
                {
                    r[0] = 0xaa55aa55aa55aa55u64;
                }
                pc = 0x1400caeca;
            }
            0x1400caeca => {
                // 1400caeca  4985fd  TEST R13,RDI
                {
                    let v = r[13] & r[7];
                    f.logic(v, 64);
                }
                // 1400caecd  7415  JZ 0x1400caee4
                pc = if f.z { 0x1400caee4 } else { 0x1400caecf };
                continue;
            }
            0x1400caecf => {
                // 1400caecf  452bc2  SUB R8D,R10D
                {
                    let v = f.arithmetic(r[8] & 0xffffffffu64, r[10] & 0xffffffffu64, 32, true);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 1400caed2  418bc0  MOV EAX,R8D
                {
                    r[0] = (r[8] & 0xffffffffu64) & 0xffff_ffff;
                }
                // 1400caed5  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400caed6  33c2  XOR EAX,EDX
                {
                    let v = (r[0] & 0xffffffffu64 ^ r[2] & 0xffffffffu64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400caed8  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400caeda  69c002000200  IMUL EAX,EAX,0x20002
                {
                    r[0] = ((r[0] & 0xffffffffu64).wrapping_mul(0x20002u64)) & 0xffff_ffff;
                }
                // 1400caee0  03c8  ADD ECX,EAX
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                // 1400caee2  eb37  JMP 0x1400caf1b
                pc = 0x1400caf1b;
                continue;
            }
            0x1400caee4 => {
                // 1400caee4  4c85fa  TEST RDX,R15
                {
                    let v = r[2] & r[15];
                    f.logic(v, 64);
                }
                // 1400caee7  7415  JZ 0x1400caefe
                pc = if f.z { 0x1400caefe } else { 0x1400caee9 };
                continue;
            }
            0x1400caee9 => {
                // 1400caee9  4885f6  TEST RSI,RSI
                {
                    let v = r[6];
                    f.logic(v, 64);
                }
                // 1400caeec  7507  JNZ 0x1400caef5
                pc = if !f.z { 0x1400caef5 } else { 0x1400caeee };
                continue;
            }
            0x1400caeee => {
                // 1400caeee  4469e308000800  IMUL R12D,EBX,0x80008
                {
                    r[12] = ((r[3] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                pc = 0x1400caef5;
            }
            0x1400caef5 => {
                // 1400caef5  4169c108000800  IMUL EAX,R9D,0x80008
                {
                    r[0] = ((r[9] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                // 1400caefc  eb17  JMP 0x1400caf15
                pc = 0x1400caf15;
                continue;
            }
            0x1400caefe => {
                // 1400caefe  4c85f8  TEST RAX,R15
                {
                    let v = r[0] & r[15];
                    f.logic(v, 64);
                }
                // 1400caf01  7418  JZ 0x1400caf1b
                pc = if f.z { 0x1400caf1b } else { 0x1400caf03 };
                continue;
            }
            0x1400caf03 => {
                // 1400caf03  4885f6  TEST RSI,RSI
                {
                    let v = r[6];
                    f.logic(v, 64);
                }
                // 1400caf06  7507  JNZ 0x1400caf0f
                pc = if !f.z { 0x1400caf0f } else { 0x1400caf08 };
                continue;
            }
            0x1400caf08 => {
                // 1400caf08  4569e108000800  IMUL R12D,R9D,0x80008
                {
                    r[12] = ((r[9] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                pc = 0x1400caf0f;
            }
            0x1400caf0f => {
                // 1400caf0f  69c308000800  IMUL EAX,EBX,0x80008
                {
                    r[0] = ((r[3] & 0xffffffffu64).wrapping_mul(0x80008u64)) & 0xffff_ffff;
                }
                pc = 0x1400caf15;
            }
            0x1400caf15 => {
                // 1400caf15  442be0  SUB R12D,EAX
                {
                    let v = f.arithmetic(r[12] & 0xffffffffu64, r[0] & 0xffffffffu64, 32, true);
                    r[12] = (v) & 0xffff_ffff;
                }
                // 1400caf18  4103cc  ADD ECX,R12D
                {
                    let v = f.arithmetic(r[1] & 0xffffffffu64, r[12] & 0xffffffffu64, 32, false);
                    r[1] = (v) & 0xffff_ffff;
                }
                pc = 0x1400caf1b;
            }
            0x1400caf1b => {
                // 1400caf1b  660f6f05ed8c1c00  MOVDQA XMM0,xmmword ptr [0x140293c10]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293c10u64));
                }
                // 1400caf23  4c8d35d650f3ff  LEA R14,[0x140000000]
                {
                    r[14] = 0u64.wrapping_add(0x140000000u64);
                }
                // 1400caf2a  660f6f0dee8c1c00  MOVDQA XMM1,xmmword ptr [0x140293c20]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293c20u64));
                }
                // 1400caf32  48be0202020202020202  MOV RSI,0x202020202020202
                {
                    r[6] = 0x202020202020202u64;
                }
                // 1400caf3c  4c8b052d622f00  MOV R8,qword ptr [0x1403c1170]
                {
                    r[8] = m.read::<64>(0u64.wrapping_add(0x1403c1170u64));
                }
                // 1400caf43  bf000000e0  MOV EDI,0xe0000000
                {
                    r[7] = (0xe0000000u64) & 0xffff_ffff;
                }
                // 1400caf48  488b15c1612f00  MOV RDX,qword ptr [0x1403c1110]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400caf4f  49ba0101010101010101  MOV R10,0x101010101010101
                {
                    r[10] = 0x101010101010101u64;
                }
                // 1400caf59  660f7f442430  MOVDQA xmmword ptr [RSP + 0x30],XMM0
                {
                    m.write128(0u64.wrapping_add(r[4]).wrapping_add(0x30u64), x[0]);
                }
                // 1400caf5f  49bd0404040404040404  MOV R13,0x404040404040404
                {
                    r[13] = 0x404040404040404u64;
                }
                // 1400caf69  660f6f052f8d1c00  MOVDQA XMM0,xmmword ptr [0x140293ca0]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293ca0u64));
                }
                // 1400caf71  49bc0808080808080808  MOV R12,0x808080808080808
                {
                    r[12] = 0x808080808080808u64;
                }
                // 1400caf7b  660f7f4c2440  MOVDQA xmmword ptr [RSP + 0x40],XMM1
                {
                    m.write128(0u64.wrapping_add(r[4]).wrapping_add(0x40u64), x[1]);
                }
                // 1400caf81  49bf1010101010101010  MOV R15,0x1010101010101010
                {
                    r[15] = 0x1010101010101010u64;
                }
                // 1400caf8b  660f6f0d1d8d1c00  MOVDQA XMM1,xmmword ptr [0x140293cb0]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293cb0u64));
                }
                // 1400caf93  660f7f442450  MOVDQA xmmword ptr [RSP + 0x50],XMM0
                {
                    m.write128(0u64.wrapping_add(r[4]).wrapping_add(0x50u64), x[0]);
                }
                // 1400caf99  660f6f05bf8c1c00  MOVDQA XMM0,xmmword ptr [0x140293c60]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293c60u64));
                }
                // 1400cafa1  660f7f4c2460  MOVDQA xmmword ptr [RSP + 0x60],XMM1
                {
                    m.write128(0u64.wrapping_add(r[4]).wrapping_add(0x60u64), x[1]);
                }
                // 1400cafa7  660f6f0dc18c1c00  MOVDQA XMM1,xmmword ptr [0x140293c70]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293c70u64));
                }
                // 1400cafaf  660f7f442470  MOVDQA xmmword ptr [RSP + 0x70],XMM0
                {
                    m.write128(0u64.wrapping_add(r[4]).wrapping_add(0x70u64), x[0]);
                }
                // 1400cafb5  660f6f05238d1c00  MOVDQA XMM0,xmmword ptr [0x140293ce0]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293ce0u64));
                }
                // 1400cafbd  660f7f4d80  MOVDQA xmmword ptr [RBP + -0x80],XMM1
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xffffffffffffff80u64),
                        x[1],
                    );
                }
                // 1400cafc2  660f6f0d268d1c00  MOVDQA XMM1,xmmword ptr [0x140293cf0]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293cf0u64));
                }
                // 1400cafca  660f7f4590  MOVDQA xmmword ptr [RBP + -0x70],XMM0
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xffffffffffffff90u64),
                        x[0],
                    );
                }
                // 1400cafcf  660f6f05698c1c00  MOVDQA XMM0,xmmword ptr [0x140293c40]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293c40u64));
                }
                // 1400cafd7  660f7f4da0  MOVDQA xmmword ptr [RBP + -0x60],XMM1
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xffffffffffffffa0u64),
                        x[1],
                    );
                }
                // 1400cafdc  660f6f0d6c8c1c00  MOVDQA XMM1,xmmword ptr [0x140293c50]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293c50u64));
                }
                // 1400cafe4  660f7f45b0  MOVDQA xmmword ptr [RBP + -0x50],XMM0
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xffffffffffffffb0u64),
                        x[0],
                    );
                }
                // 1400cafe9  660f6f05cf8c1c00  MOVDQA XMM0,xmmword ptr [0x140293cc0]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293cc0u64));
                }
                // 1400caff1  660f7f4dc0  MOVDQA xmmword ptr [RBP + -0x40],XMM1
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xffffffffffffffc0u64),
                        x[1],
                    );
                }
                // 1400caff6  660f6f0dd28c1c00  MOVDQA XMM1,xmmword ptr [0x140293cd0]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293cd0u64));
                }
                // 1400caffe  4963c0  MOVSXD RAX,R8D
                {
                    r[0] = ((r[8] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cb001  660f7f45d0  MOVDQA xmmword ptr [RBP + -0x30],XMM0
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xffffffffffffffd0u64),
                        x[0],
                    );
                }
                // 1400cb006  660f6f05728c1c00  MOVDQA XMM0,xmmword ptr [0x140293c80]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293c80u64));
                }
                // 1400cb00e  660f7f4de0  MOVDQA xmmword ptr [RBP + -0x20],XMM1
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xffffffffffffffe0u64),
                        x[1],
                    );
                }
                // 1400cb013  660f6f0d758c1c00  MOVDQA XMM1,xmmword ptr [0x140293c90]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293c90u64));
                }
                // 1400cb01b  498b9cc6803e0f00  MOV RBX,qword ptr [R14 + RAX*0x8 + 0xf3e80]
                {
                    r[3] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xf3e80u64),
                    );
                }
                // 1400cb023  460fb68c30000a0f00  MOVZX R9D,byte ptr [RAX + R14*0x1 + 0xf0a00]
                {
                    r[9] = (m.read::<8>(
                        0u64.wrapping_add(r[0])
                            .wrapping_add(r[14].wrapping_mul(0x1u64))
                            .wrapping_add(0xf0a00u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb02c  48f7d3  NOT RBX
                {
                    let v = !(r[3]);
                    r[3] = v;
                }
                // 1400cb02f  660f7f45f0  MOVDQA xmmword ptr [RBP + -0x10],XMM0
                {
                    m.write128(
                        0u64.wrapping_add(r[5]).wrapping_add(0xfffffffffffffff0u64),
                        x[0],
                    );
                }
                // 1400cb034  4823da  AND RBX,RDX
                {
                    let v = r[3] & r[2];
                    f.logic(v, 64);
                    r[3] = v;
                }
                // 1400cb037  660f6f05c18c1c00  MOVDQA XMM0,xmmword ptr [0x140293d00]
                {
                    x[0] = m.read128(0u64.wrapping_add(0x140293d00u64));
                }
                // 1400cb03f  660f7f4d00  MOVDQA xmmword ptr [RBP],XMM1
                {
                    m.write128(0u64.wrapping_add(r[5]), x[1]);
                }
                // 1400cb044  660f6f0dc48c1c00  MOVDQA XMM1,xmmword ptr [0x140293d10]
                {
                    x[1] = m.read128(0u64.wrapping_add(0x140293d10u64));
                }
                // 1400cb04c  660f7f4510  MOVDQA xmmword ptr [RBP + 0x10],XMM0
                {
                    m.write128(0u64.wrapping_add(r[5]).wrapping_add(0x10u64), x[0]);
                }
                // 1400cb051  660f7f4d20  MOVDQA xmmword ptr [RBP + 0x20],XMM1
                {
                    m.write128(0u64.wrapping_add(r[5]).wrapping_add(0x20u64), x[1]);
                }
                // 1400cb056  4584c9  TEST R9B,R9B
                {
                    let v = (r[9] & 0xffu64) & 0xffu64;
                    f.logic(v, 8);
                }
                // 1400cb059  0f85bc000000  JNZ 0x1400cb11b
                pc = if !f.z { 0x1400cb11b } else { 0x1400cb05f };
                continue;
            }
            0x1400cb05f => {
                // 1400cb05f  488b15b2602f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb066  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb069  2500000700  AND EAX,0x70000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x70000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb06e  4c8bc2  MOV R8,RDX
                {
                    r[8] = r[2];
                }
                // 1400cb071  49c1e805  SHR R8,0x5
                {
                    let v = r[8];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb075  4c8bcb  MOV R9,RBX
                {
                    r[9] = r[3];
                }
                // 1400cb078  49c1e908  SHR R9,0x8
                {
                    let v = r[9];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb07c  4181e000000038  AND R8D,0x38000000
                {
                    let v = (r[8] & 0xffffffffu64 & 0x38000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 1400cb083  4c0bcb  OR R9,RBX
                {
                    let v = r[9] | r[3];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb086  81e300070000  AND EBX,0x700
                {
                    let v = (r[3] & 0xffffffffu64 & 0x700u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[3] = (v) & 0xffff_ffff;
                }
                // 1400cb08c  49c1e905  SHR R9,0x5
                {
                    let v = r[9];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb090  4181e100003800  AND R9D,0x380000
                {
                    let v = (r[9] & 0xffffffffu64 & 0x380000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400cb097  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb09a  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400cb09d  2500000007  AND EAX,0x7000000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb0a2  49c1e905  SHR R9,0x5
                {
                    let v = r[9];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb0a6  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb0a9  81e200000700  AND EDX,0x70000
                {
                    let v = (r[2] & 0xffffffffu64 & 0x70000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cb0af  49c1e805  SHR R8,0x5
                {
                    let v = r[8];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb0b3  4c0bcb  OR R9,RBX
                {
                    let v = r[9] | r[3];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb0b6  4c0bc2  OR R8,RDX
                {
                    let v = r[8] | r[2];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb0b9  49c1e908  SHR R9,0x8
                {
                    let v = r[9];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb0bd  49c1e810  SHR R8,0x10
                {
                    let v = r[8];
                    let n = 16u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb0c1  438b8486e0121000  MOV EAX,dword ptr [R14 + R8*0x4 + 0x1012e0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[8].wrapping_mul(0x4u64))
                            .wrapping_add(0x1012e0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb0c9  4303848ee01a1000  ADD EAX,dword ptr [R14 + R9*0x4 + 0x101ae0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[9].wrapping_mul(0x4u64))
                                .wrapping_add(0x101ae0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb0d1  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb0d5  488b153c602f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb0dc  4985d2  TEST R10,RDX
                {
                    let v = r[10] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb0df  7510  JNZ 0x1400cb0f1
                pc = if !f.z { 0x1400cb0f1 } else { 0x1400cb0e1 };
                continue;
            }
            0x1400cb0e1 => {
                // 1400cb0e1  0500004600  ADD EAX,0x460000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x460000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb0e6  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb0ea  488b1527602f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x1400cb0f1;
            }
            0x1400cb0f1 => {
                // 1400cb0f1  4885d6  TEST RSI,RDX
                {
                    let v = r[6] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb0f4  750f  JNZ 0x1400cb105
                pc = if !f.z { 0x1400cb105 } else { 0x1400cb0f6 };
                continue;
            }
            0x1400cb0f6 => {
                // 1400cb0f6  4181433400004600  ADD dword ptr [R11 + 0x34],0x460000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x460000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                // 1400cb0fe  488b1513602f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x1400cb105;
            }
            0x1400cb105 => {
                // 1400cb105  4985d5  TEST R13,RDX
                {
                    let v = r[13] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb108  0f8572020000  JNZ 0x1400cb380
                pc = if !f.z { 0x1400cb380 } else { 0x1400cb10e };
                continue;
            }
            0x1400cb10e => {
                // 1400cb10e  4181433400003200  ADD dword ptr [R11 + 0x34],0x320000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x320000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                // 1400cb116  e95e020000  JMP 0x1400cb379
                pc = 0x1400cb379;
                continue;
            }
            0x1400cb11b => {
                // 1400cb11b  4180f902  CMP R9B,0x2
                {
                    f.arithmetic(r[9] & 0xffu64, 0x2u64, 8, true);
                }
                // 1400cb11f  0f85f8000000  JNZ 0x1400cb21d
                pc = if !f.z { 0x1400cb21d } else { 0x1400cb125 };
                continue;
            }
            0x1400cb125 => {
                // 1400cb125  4c8b0dec5f2f00  MOV R9,qword ptr [0x1403c1118]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb12c  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb12f  2500e00000  AND EAX,0xe000
                {
                    let v = (r[0] & 0xffffffffu64 & 0xe000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb134  488bd3  MOV RDX,RBX
                {
                    r[2] = r[3];
                }
                // 1400cb137  48c1ea05  SHR RDX,0x5
                {
                    let v = r[2];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb13b  81e200000700  AND EDX,0x70000
                {
                    let v = (r[2] & 0xffffffffu64 & 0x70000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cb141  480bd0  OR RDX,RAX
                {
                    let v = r[2] | r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cb144  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb147  48c1e808  SHR RAX,0x8
                {
                    let v = r[0];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400cb14b  480bc3  OR RAX,RBX
                {
                    let v = r[0] | r[3];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb14e  48c1ea0d  SHR RDX,0xd
                {
                    let v = r[2];
                    let n = 13u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb152  48c1e81d  SHR RAX,0x1d
                {
                    let v = r[0];
                    let n = 29u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400cb156  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb159  4c63549430  MOVSXD R10,dword ptr [RSP + RDX*0x4 + 0x30]
                {
                    r[10] = ((m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[2].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) as i32 as i64) as u64;
                }
                // 1400cb15e  498bd1  MOV RDX,R9
                {
                    r[2] = r[9];
                }
                // 1400cb161  48c1ea05  SHR RDX,0x5
                {
                    let v = r[2];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb165  8b448430  MOV EAX,dword ptr [RSP + RAX*0x4 + 0x30]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb169  c1e006  SHL EAX,0x6
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 6u32;
                    if n != 0 {
                        let v = (v.wrapping_shl(n)) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cb16c  4898  CDQE
                {
                    r[0] = r[0] as i32 as i64 as u64;
                }
                // 1400cb16e  4c0bd0  OR R10,RAX
                {
                    let v = r[10] | r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 1400cb171  48b80000000007000000  MOV RAX,0x700000000
                {
                    r[0] = 0x700000000u64;
                }
                // 1400cb17b  4823d0  AND RDX,RAX
                {
                    let v = r[2] & r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cb17e  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 1400cb181  4823c7  AND RAX,RDI
                {
                    let v = r[0] & r[7];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb184  49c1e915  SHR R9,0x15
                {
                    let v = r[9];
                    let n = 21u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb188  480bd0  OR RDX,RAX
                {
                    let v = r[2] | r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cb18b  4183e107  AND R9D,0x7
                {
                    let v = (r[9] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400cb18f  48c1ea1d  SHR RDX,0x1d
                {
                    let v = r[2];
                    let n = 29u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb193  8b449430  MOV EAX,dword ptr [RSP + RDX*0x4 + 0x30]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[2].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb197  c1e003  SHL EAX,0x3
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 3u32;
                    if n != 0 {
                        let v = (v.wrapping_shl(n)) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cb19a  4c63c0  MOVSXD R8,EAX
                {
                    r[8] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cb19d  4a63448c30  MOVSXD RAX,dword ptr [RSP + R9*0x4 + 0x30]
                {
                    r[0] = ((m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[9].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) as i32 as i64) as u64;
                }
                // 1400cb1a2  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb1a5  438b8486e0121000  MOV EAX,dword ptr [R14 + R8*0x4 + 0x1012e0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[8].wrapping_mul(0x4u64))
                            .wrapping_add(0x1012e0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb1ad  49b88080808080808080  MOV R8,-0x7f7f7f7f7f7f7f80
                {
                    r[8] = 0x8080808080808080u64;
                }
                // 1400cb1b7  43038496e01a1000  ADD EAX,dword ptr [R14 + R10*0x4 + 0x101ae0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[10].wrapping_mul(0x4u64))
                                .wrapping_add(0x101ae0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb1bf  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb1c3  488b154e5f2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb1ca  4985d0  TEST R8,RDX
                {
                    let v = r[8] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb1cd  7510  JNZ 0x1400cb1df
                pc = if !f.z { 0x1400cb1df } else { 0x1400cb1cf };
                continue;
            }
            0x1400cb1cf => {
                // 1400cb1cf  0500004600  ADD EAX,0x460000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x460000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb1d4  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb1d8  488b15395f2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x1400cb1df;
            }
            0x1400cb1df => {
                // 1400cb1df  48b84040404040404040  MOV RAX,0x4040404040404040
                {
                    r[0] = 0x4040404040404040u64;
                }
                // 1400cb1e9  4885d0  TEST RAX,RDX
                {
                    let v = r[0] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb1ec  750f  JNZ 0x1400cb1fd
                pc = if !f.z { 0x1400cb1fd } else { 0x1400cb1ee };
                continue;
            }
            0x1400cb1ee => {
                // 1400cb1ee  4181433400004600  ADD dword ptr [R11 + 0x34],0x460000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x460000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                // 1400cb1f6  488b151b5f2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x1400cb1fd;
            }
            0x1400cb1fd => {
                // 1400cb1fd  48b82020202020202020  MOV RAX,0x2020202020202020
                {
                    r[0] = 0x2020202020202020u64;
                }
                // 1400cb207  4885d0  TEST RAX,RDX
                {
                    let v = r[0] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb20a  0f8570010000  JNZ 0x1400cb380
                pc = if !f.z { 0x1400cb380 } else { 0x1400cb210 };
                continue;
            }
            0x1400cb210 => {
                // 1400cb210  4181433400003200  ADD dword ptr [R11 + 0x34],0x320000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x320000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                // 1400cb218  e95c010000  JMP 0x1400cb379
                pc = 0x1400cb379;
                continue;
            }
            0x1400cb21d => {
                // 1400cb21d  4183e007  AND R8D,0x7
                {
                    let v = (r[8] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 1400cb221  4180f803  CMP R8B,0x3
                {
                    f.arithmetic(r[8] & 0xffu64, 0x3u64, 8, true);
                }
                // 1400cb225  7560  JNZ 0x1400cb287
                pc = if !f.z { 0x1400cb287 } else { 0x1400cb227 };
                continue;
            }
            0x1400cb227 => {
                // 1400cb227  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400cb22a  4c8bc2  MOV R8,RDX
                {
                    r[8] = r[2];
                }
                // 1400cb22d  49c1e808  SHR R8,0x8
                {
                    let v = r[8];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb231  2500001c00  AND EAX,0x1c0000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x1c0000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb236  4c0bc2  OR R8,RDX
                {
                    let v = r[8] | r[2];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb239  81e2001c0000  AND EDX,0x1c00
                {
                    let v = (r[2] & 0xffffffffu64 & 0x1c00u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cb23f  49c1e805  SHR R8,0x5
                {
                    let v = r[8];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb243  4181e00000e000  AND R8D,0xe00000
                {
                    let v = (r[8] & 0xffffffffu64 & 0xe00000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 1400cb24a  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb24d  49c1e805  SHR R8,0x5
                {
                    let v = r[8];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb251  4c0bc2  OR R8,RDX
                {
                    let v = r[8] | r[2];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb254  488b15bd5e2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb25b  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400cb25e  49c1e80a  SHR R8,0xa
                {
                    let v = r[8];
                    let n = 10u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb262  4c8bca  MOV R9,RDX
                {
                    r[9] = r[2];
                }
                // 1400cb265  250000001c  AND EAX,0x1c000000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x1c000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb26a  49c1e905  SHR R9,0x5
                {
                    let v = r[9];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb26e  81e200001c00  AND EDX,0x1c0000
                {
                    let v = (r[2] & 0xffffffffu64 & 0x1c0000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cb274  4c23cf  AND R9,RDI
                {
                    let v = r[9] & r[7];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb277  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb27a  49c1e905  SHR R9,0x5
                {
                    let v = r[9];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb27e  4c0bca  OR R9,RDX
                {
                    let v = r[9] | r[2];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb281  49c1e912  SHR R9,0x12
                {
                    let v = r[9];
                    let n = 18u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb285  eb68  JMP 0x1400cb2ef
                pc = 0x1400cb2ef;
                continue;
            }
            0x1400cb287 => {
                // 1400cb287  488b158a5e2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb28e  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb291  2500003800  AND EAX,0x380000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x380000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb296  4c8bc3  MOV R8,RBX
                {
                    r[8] = r[3];
                }
                // 1400cb299  49c1e808  SHR R8,0x8
                {
                    let v = r[8];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb29d  4c8bca  MOV R9,RDX
                {
                    r[9] = r[2];
                }
                // 1400cb2a0  4c0bc3  OR R8,RBX
                {
                    let v = r[8] | r[3];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb2a3  49c1e905  SHR R9,0x5
                {
                    let v = r[9];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb2a7  49c1e805  SHR R8,0x5
                {
                    let v = r[8];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb2ab  81e300380000  AND EBX,0x3800
                {
                    let v = (r[3] & 0xffffffffu64 & 0x3800u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[3] = (v) & 0xffff_ffff;
                }
                // 1400cb2b1  4181e00000c001  AND R8D,0x1c00000
                {
                    let v = (r[8] & 0xffffffffu64 & 0x1c00000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[8] = (v) & 0xffff_ffff;
                }
                // 1400cb2b8  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb2bb  48b8000000c001000000  MOV RAX,0x1c0000000
                {
                    r[0] = 0x1c0000000u64;
                }
                // 1400cb2c5  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb2c8  49c1e805  SHR R8,0x5
                {
                    let v = r[8];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb2cc  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400cb2cf  4c0bc3  OR R8,RBX
                {
                    let v = r[8] | r[3];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb2d2  2500000038  AND EAX,0x38000000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x38000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb2d7  49c1e80b  SHR R8,0xb
                {
                    let v = r[8];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb2db  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb2de  81e200003800  AND EDX,0x380000
                {
                    let v = (r[2] & 0xffffffffu64 & 0x380000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cb2e4  49c1e905  SHR R9,0x5
                {
                    let v = r[9];
                    let n = 5u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb2e8  4c0bca  OR R9,RDX
                {
                    let v = r[9] | r[2];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb2eb  49c1e913  SHR R9,0x13
                {
                    let v = r[9];
                    let n = 19u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                pc = 0x1400cb2ef;
            }
            0x1400cb2ef => {
                // 1400cb2ef  f6058a5e2f0003  TEST byte ptr [0x1403c1180],0x3
                {
                    let v = (m.read::<8>(0u64.wrapping_add(0x1403c1180u64)) & 0x3u64) & 0xffu64;
                    f.logic(v, 8);
                }
                // 1400cb2f6  7444  JZ 0x1400cb33c
                pc = if f.z { 0x1400cb33c } else { 0x1400cb2f8 };
                continue;
            }
            0x1400cb2f8 => {
                // 1400cb2f8  438b848ee0121000  MOV EAX,dword ptr [R14 + R9*0x4 + 0x1012e0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[9].wrapping_mul(0x4u64))
                            .wrapping_add(0x1012e0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb300  43038486e01a1000  ADD EAX,dword ptr [R14 + R8*0x4 + 0x101ae0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[8].wrapping_mul(0x4u64))
                                .wrapping_add(0x101ae0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb308  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400cb309  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb30b  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cb30d  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb311  488b15005e2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb318  4985d4  TEST R12,RDX
                {
                    let v = r[12] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb31b  7510  JNZ 0x1400cb32d
                pc = if !f.z { 0x1400cb32d } else { 0x1400cb31d };
                continue;
            }
            0x1400cb31d => {
                // 1400cb31d  0500002300  ADD EAX,0x230000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x230000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb322  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb326  488b15eb5d2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x1400cb32d;
            }
            0x1400cb32d => {
                // 1400cb32d  4985d7  TEST R15,RDX
                {
                    let v = r[15] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb330  754e  JNZ 0x1400cb380
                pc = if !f.z { 0x1400cb380 } else { 0x1400cb332 };
                continue;
            }
            0x1400cb332 => {
                // 1400cb332  4181433400002300  ADD dword ptr [R11 + 0x34],0x230000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x230000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                // 1400cb33a  eb3d  JMP 0x1400cb379
                pc = 0x1400cb379;
                continue;
            }
            0x1400cb33c => {
                // 1400cb33c  438b8486e01a1000  MOV EAX,dword ptr [R14 + R8*0x4 + 0x101ae0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[8].wrapping_mul(0x4u64))
                            .wrapping_add(0x101ae0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb344  4303848ee0121000  ADD EAX,dword ptr [R14 + R9*0x4 + 0x1012e0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[9].wrapping_mul(0x4u64))
                                .wrapping_add(0x1012e0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb34c  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb350  488b15c15d2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                // 1400cb357  4985d4  TEST R12,RDX
                {
                    let v = r[12] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb35a  7510  JNZ 0x1400cb36c
                pc = if !f.z { 0x1400cb36c } else { 0x1400cb35c };
                continue;
            }
            0x1400cb35c => {
                // 1400cb35c  0500004600  ADD EAX,0x460000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x460000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb361  41894334  MOV dword ptr [R11 + 0x34],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x34u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb365  488b15ac5d2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x1400cb36c;
            }
            0x1400cb36c => {
                // 1400cb36c  4985d7  TEST R15,RDX
                {
                    let v = r[15] & r[2];
                    f.logic(v, 64);
                }
                // 1400cb36f  750f  JNZ 0x1400cb380
                pc = if !f.z { 0x1400cb380 } else { 0x1400cb371 };
                continue;
            }
            0x1400cb371 => {
                // 1400cb371  4181433400004600  ADD dword ptr [R11 + 0x34],0x460000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x460000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                pc = 0x1400cb379;
            }
            0x1400cb379 => {
                // 1400cb379  488b15985d2f00  MOV RDX,qword ptr [0x1403c1118]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1118u64));
                }
                pc = 0x1400cb380;
            }
            0x1400cb380 => {
                // 1400cb380  48633ded5d2f00  MOVSXD RDI,dword ptr [0x1403c1174]
                {
                    r[7] = ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 1400cb387  498b9cfe80400f00  MOV RBX,qword ptr [R14 + RDI*0x8 + 0xf4080]
                {
                    r[3] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[7].wrapping_mul(0x8u64))
                            .wrapping_add(0xf4080u64),
                    );
                }
                // 1400cb38f  460fb68437000a0f00  MOVZX R8D,byte ptr [RDI + R14*0x1 + 0xf0a00]
                {
                    r[8] = (m.read::<8>(
                        0u64.wrapping_add(r[7])
                            .wrapping_add(r[14].wrapping_mul(0x1u64))
                            .wrapping_add(0xf0a00u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb398  48f7d3  NOT RBX
                {
                    let v = !(r[3]);
                    r[3] = v;
                }
                // 1400cb39b  4823da  AND RBX,RDX
                {
                    let v = r[3] & r[2];
                    f.logic(v, 64);
                    r[3] = v;
                }
                // 1400cb39e  4584c0  TEST R8B,R8B
                {
                    let v = (r[8] & 0xffu64) & 0xffu64;
                    f.logic(v, 8);
                }
                // 1400cb3a1  0f85d6000000  JNZ 0x1400cb47d
                pc = if !f.z { 0x1400cb47d } else { 0x1400cb3a7 };
                continue;
            }
            0x1400cb3a7 => {
                // 1400cb3a7  488b15625d2f00  MOV RDX,qword ptr [0x1403c1110]
                {
                    r[2] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400cb3ae  4c8bcb  MOV R9,RBX
                {
                    r[9] = r[3];
                }
                // 1400cb3b1  49c1e90b  SHR R9,0xb
                {
                    let v = r[9];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb3b5  4c8bc2  MOV R8,RDX
                {
                    r[8] = r[2];
                }
                // 1400cb3b8  49c1e80b  SHR R8,0xb
                {
                    let v = r[8];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb3bc  48b800000000e0000000  MOV RAX,0xe000000000
                {
                    r[0] = 0xe000000000u64;
                }
                // 1400cb3c6  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb3c9  49ba0000000000070000  MOV R10,0x70000000000
                {
                    r[10] = 0x70000000000u64;
                }
                // 1400cb3d3  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb3d6  4923c2  AND RAX,R10
                {
                    let v = r[0] & r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb3d9  49ba0000000007000000  MOV R10,0x700000000
                {
                    r[10] = 0x700000000u64;
                }
                // 1400cb3e3  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb3e6  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb3e9  48c1e808  SHR RAX,0x8
                {
                    let v = r[0];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400cb3ed  480bc3  OR RAX,RBX
                {
                    let v = r[0] | r[3];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb3f0  49c1e913  SHR R9,0x13
                {
                    let v = r[9];
                    let n = 19u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb3f4  2500000007  AND EAX,0x7000000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb3f9  bb000000e0  MOV EBX,0xe0000000
                {
                    r[3] = (0xe0000000u64) & 0xffff_ffff;
                }
                // 1400cb3fe  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb401  4c23c3  AND R8,RBX
                {
                    let v = r[8] & r[3];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb404  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400cb407  49c1e912  SHR R9,0x12
                {
                    let v = r[9];
                    let n = 18u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb40b  4923c2  AND RAX,R10
                {
                    let v = r[0] & r[10];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb40e  81e200000007  AND EDX,0x7000000
                {
                    let v = (r[2] & 0xffffffffu64 & 0x7000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cb414  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb417  49c1e80b  SHR R8,0xb
                {
                    let v = r[8];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb41b  4c0bc2  OR R8,RDX
                {
                    let v = r[8] | r[2];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb41e  48ba0101010101010101  MOV RDX,0x101010101010101
                {
                    r[2] = 0x101010101010101u64;
                }
                // 1400cb428  49c1e812  SHR R8,0x12
                {
                    let v = r[8];
                    let n = 18u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb42c  438b8486e0121000  MOV EAX,dword ptr [R14 + R8*0x4 + 0x1012e0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[8].wrapping_mul(0x4u64))
                            .wrapping_add(0x1012e0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb434  4303848ee01a1000  ADD EAX,dword ptr [R14 + R9*0x4 + 0x101ae0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[9].wrapping_mul(0x4u64))
                                .wrapping_add(0x101ae0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb43c  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb440  488515c95c2f00  TEST qword ptr [0x1403c1110],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb447  7509  JNZ 0x1400cb452
                pc = if !f.z { 0x1400cb452 } else { 0x1400cb449 };
                continue;
            }
            0x1400cb449 => {
                // 1400cb449  0500004600  ADD EAX,0x460000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x460000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb44e  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                pc = 0x1400cb452;
            }
            0x1400cb452 => {
                // 1400cb452  488535b75c2f00  TEST qword ptr [0x1403c1110],RSI
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[6];
                    f.logic(v, 64);
                }
                // 1400cb459  7508  JNZ 0x1400cb463
                pc = if !f.z { 0x1400cb463 } else { 0x1400cb45b };
                continue;
            }
            0x1400cb45b => {
                // 1400cb45b  4181433800004600  ADD dword ptr [R11 + 0x38],0x460000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x460000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                pc = 0x1400cb463;
            }
            0x1400cb463 => {
                // 1400cb463  4c852da65c2f00  TEST qword ptr [0x1403c1110],R13
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[13];
                    f.logic(v, 64);
                }
                // 1400cb46a  0f8592020000  JNZ 0x1400cb702
                pc = if !f.z { 0x1400cb702 } else { 0x1400cb470 };
                continue;
            }
            0x1400cb470 => {
                // 1400cb470  4181433800003200  ADD dword ptr [R11 + 0x38],0x320000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x320000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                // 1400cb478  e985020000  JMP 0x1400cb702
                pc = 0x1400cb702;
                continue;
            }
            0x1400cb47d => {
                // 1400cb47d  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb480  4180f802  CMP R8B,0x2
                {
                    f.arithmetic(r[8] & 0xffu64, 0x2u64, 8, true);
                }
                // 1400cb484  0f85f9000000  JNZ 0x1400cb583
                pc = if !f.z { 0x1400cb583 } else { 0x1400cb48a };
                continue;
            }
            0x1400cb48a => {
                // 1400cb48a  488bd3  MOV RDX,RBX
                {
                    r[2] = r[3];
                }
                // 1400cb48d  49b900000000001c0000  MOV R9,0x1c0000000000
                {
                    r[9] = 0x1c0000000000u64;
                }
                // 1400cb497  48c1ea0b  SHR RDX,0xb
                {
                    let v = r[2];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb49b  49b80000000000e00000  MOV R8,0xe00000000000
                {
                    r[8] = 0xe00000000000u64;
                }
                // 1400cb4a5  4923c0  AND RAX,R8
                {
                    let v = r[0] & r[8];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb4a8  4923d1  AND RDX,R9
                {
                    let v = r[2] & r[9];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cb4ab  4c8b0d5e5c2f00  MOV R9,qword ptr [0x1403c1110]
                {
                    r[9] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400cb4b2  480bd0  OR RDX,RAX
                {
                    let v = r[2] | r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cb4b5  48c1ea2a  SHR RDX,0x2a
                {
                    let v = r[2];
                    let n = 42u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb4b9  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb4bc  48c1e808  SHR RAX,0x8
                {
                    let v = r[0];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400cb4c0  480bc3  OR RAX,RBX
                {
                    let v = r[0] | r[3];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb4c3  bb000000e0  MOV EBX,0xe0000000
                {
                    r[3] = (0xe0000000u64) & 0xffff_ffff;
                }
                // 1400cb4c8  48c1e81d  SHR RAX,0x1d
                {
                    let v = r[0];
                    let n = 29u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400cb4cc  4c63549430  MOVSXD R10,dword ptr [RSP + RDX*0x4 + 0x30]
                {
                    r[10] = ((m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[2].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) as i32 as i64) as u64;
                }
                // 1400cb4d1  83e007  AND EAX,0x7
                {
                    let v = (r[0] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb4d4  498bd1  MOV RDX,R9
                {
                    r[2] = r[9];
                }
                // 1400cb4d7  48c1ea0b  SHR RDX,0xb
                {
                    let v = r[2];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb4db  81e20000001c  AND EDX,0x1c000000
                {
                    let v = (r[2] & 0xffffffffu64 & 0x1c000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[2] = (v) & 0xffff_ffff;
                }
                // 1400cb4e1  8b448430  MOV EAX,dword ptr [RSP + RAX*0x4 + 0x30]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[0].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb4e5  c1e006  SHL EAX,0x6
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 6u32;
                    if n != 0 {
                        let v = (v.wrapping_shl(n)) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cb4e8  4898  CDQE
                {
                    r[0] = r[0] as i32 as i64 as u64;
                }
                // 1400cb4ea  4c0bd0  OR R10,RAX
                {
                    let v = r[10] | r[0];
                    f.logic(v, 64);
                    r[10] = v;
                }
                // 1400cb4ed  498bc1  MOV RAX,R9
                {
                    r[0] = r[9];
                }
                // 1400cb4f0  4823c3  AND RAX,RBX
                {
                    let v = r[0] & r[3];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb4f3  49c1e92d  SHR R9,0x2d
                {
                    let v = r[9];
                    let n = 45u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb4f7  480bd0  OR RDX,RAX
                {
                    let v = r[2] | r[0];
                    f.logic(v, 64);
                    r[2] = v;
                }
                // 1400cb4fa  4183e107  AND R9D,0x7
                {
                    let v = (r[9] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[9] = (v) & 0xffff_ffff;
                }
                // 1400cb4fe  48c1ea1a  SHR RDX,0x1a
                {
                    let v = r[2];
                    let n = 26u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[2] = v;
                    }
                }
                // 1400cb502  8b449430  MOV EAX,dword ptr [RSP + RDX*0x4 + 0x30]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[2].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb506  48ba8080808080808080  MOV RDX,-0x7f7f7f7f7f7f7f80
                {
                    r[2] = 0x8080808080808080u64;
                }
                // 1400cb510  c1e003  SHL EAX,0x3
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 3u32;
                    if n != 0 {
                        let v = (v.wrapping_shl(n)) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cb513  4c63c0  MOVSXD R8,EAX
                {
                    r[8] = ((r[0] & 0xffffffffu64) as i32 as i64) as u64;
                }
                // 1400cb516  4a63448c30  MOVSXD RAX,dword ptr [RSP + R9*0x4 + 0x30]
                {
                    r[0] = ((m.read::<32>(
                        0u64.wrapping_add(r[4])
                            .wrapping_add(r[9].wrapping_mul(0x4u64))
                            .wrapping_add(0x30u64),
                    )) as i32 as i64) as u64;
                }
                // 1400cb51b  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb51e  438b8486e0121000  MOV EAX,dword ptr [R14 + R8*0x4 + 0x1012e0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[8].wrapping_mul(0x4u64))
                            .wrapping_add(0x1012e0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb526  43038496e01a1000  ADD EAX,dword ptr [R14 + R10*0x4 + 0x101ae0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[10].wrapping_mul(0x4u64))
                                .wrapping_add(0x101ae0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb52e  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb532  488515d75b2f00  TEST qword ptr [0x1403c1110],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb539  7509  JNZ 0x1400cb544
                pc = if !f.z { 0x1400cb544 } else { 0x1400cb53b };
                continue;
            }
            0x1400cb53b => {
                // 1400cb53b  0500004600  ADD EAX,0x460000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x460000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb540  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                pc = 0x1400cb544;
            }
            0x1400cb544 => {
                // 1400cb544  48b84040404040404040  MOV RAX,0x4040404040404040
                {
                    r[0] = 0x4040404040404040u64;
                }
                // 1400cb54e  488505bb5b2f00  TEST qword ptr [0x1403c1110],RAX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[0];
                    f.logic(v, 64);
                }
                // 1400cb555  7508  JNZ 0x1400cb55f
                pc = if !f.z { 0x1400cb55f } else { 0x1400cb557 };
                continue;
            }
            0x1400cb557 => {
                // 1400cb557  4181433800004600  ADD dword ptr [R11 + 0x38],0x460000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x460000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                pc = 0x1400cb55f;
            }
            0x1400cb55f => {
                // 1400cb55f  48b82020202020202020  MOV RAX,0x2020202020202020
                {
                    r[0] = 0x2020202020202020u64;
                }
                // 1400cb569  488505a05b2f00  TEST qword ptr [0x1403c1110],RAX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[0];
                    f.logic(v, 64);
                }
                // 1400cb570  0f858c010000  JNZ 0x1400cb702
                pc = if !f.z { 0x1400cb702 } else { 0x1400cb576 };
                continue;
            }
            0x1400cb576 => {
                // 1400cb576  4181433800003200  ADD dword ptr [R11 + 0x38],0x320000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x320000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                // 1400cb57e  e97f010000  JMP 0x1400cb702
                pc = 0x1400cb702;
                continue;
            }
            0x1400cb583 => {
                // 1400cb583  4c8bc3  MOV R8,RBX
                {
                    r[8] = r[3];
                }
                // 1400cb586  49ba0000000000070000  MOV R10,0x70000000000
                {
                    r[10] = 0x70000000000u64;
                }
                // 1400cb590  49c1e80b  SHR R8,0xb
                {
                    let v = r[8];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb594  49b90000000000380000  MOV R9,0x380000000000
                {
                    r[9] = 0x380000000000u64;
                }
                // 1400cb59e  4923c1  AND RAX,R9
                {
                    let v = r[0] & r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb5a1  4d23c2  AND R8,R10
                {
                    let v = r[8] & r[10];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb5a4  4c8b15655b2f00  MOV R10,qword ptr [0x1403c1110]
                {
                    r[10] = m.read::<64>(0u64.wrapping_add(0x1403c1110u64));
                }
                // 1400cb5ab  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb5ae  488bc3  MOV RAX,RBX
                {
                    r[0] = r[3];
                }
                // 1400cb5b1  49c1e813  SHR R8,0x13
                {
                    let v = r[8];
                    let n = 19u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb5b5  48c1e808  SHR RAX,0x8
                {
                    let v = r[0];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400cb5b9  48be0000000038000000  MOV RSI,0x3800000000
                {
                    r[6] = 0x3800000000u64;
                }
                // 1400cb5c3  480bc3  OR RAX,RBX
                {
                    let v = r[0] | r[3];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb5c6  83e707  AND EDI,0x7
                {
                    let v = (r[7] & 0xffffffffu64 & 0x7u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[7] = (v) & 0xffff_ffff;
                }
                // 1400cb5c9  2500000038  AND EAX,0x38000000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x38000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb5ce  498bda  MOV RBX,R10
                {
                    r[3] = r[10];
                }
                // 1400cb5d1  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb5d4  48c1eb0b  SHR RBX,0xb
                {
                    let v = r[3];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[3] = v;
                    }
                }
                // 1400cb5d8  48b80000000007000000  MOV RAX,0x700000000
                {
                    r[0] = 0x700000000u64;
                }
                // 1400cb5e2  49c1e815  SHR R8,0x15
                {
                    let v = r[8];
                    let n = 21u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb5e6  4c8bcb  MOV R9,RBX
                {
                    r[9] = r[3];
                }
                // 1400cb5e9  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb5ec  498bc2  MOV RAX,R10
                {
                    r[0] = r[10];
                }
                // 1400cb5ef  4823c6  AND RAX,RSI
                {
                    let v = r[0] & r[6];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb5f2  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb5f5  498bc2  MOV RAX,R10
                {
                    r[0] = r[10];
                }
                // 1400cb5f8  49c1e90b  SHR R9,0xb
                {
                    let v = r[9];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb5fc  2500000038  AND EAX,0x38000000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x38000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb601  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb604  49c1e915  SHR R9,0x15
                {
                    let v = r[9];
                    let n = 21u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb608  4080ff03  CMP DIL,0x3
                {
                    f.arithmetic(r[7] & 0xffu64, 0x3u64, 8, true);
                }
                // 1400cb60c  7576  JNZ 0x1400cb684
                pc = if !f.z { 0x1400cb684 } else { 0x1400cb60e };
                continue;
            }
            0x1400cb60e => {
                // 1400cb60e  4c8bc2  MOV R8,RDX
                {
                    r[8] = r[2];
                }
                // 1400cb611  48b80000000080030000  MOV RAX,0x38000000000
                {
                    r[0] = 0x38000000000u64;
                }
                // 1400cb61b  49c1e80b  SHR R8,0xb
                {
                    let v = r[8];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb61f  49b900000000001c0000  MOV R9,0x1c0000000000
                {
                    r[9] = 0x1c0000000000u64;
                }
                // 1400cb629  4c23c0  AND R8,RAX
                {
                    let v = r[8] & r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb62c  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400cb62f  4923c1  AND RAX,R9
                {
                    let v = r[0] & r[9];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb632  4c8bcb  MOV R9,RBX
                {
                    r[9] = r[3];
                }
                // 1400cb635  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb638  488bc2  MOV RAX,RDX
                {
                    r[0] = r[2];
                }
                // 1400cb63b  48c1e808  SHR RAX,0x8
                {
                    let v = r[0];
                    let n = 8u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[0] = v;
                    }
                }
                // 1400cb63f  480bc2  OR RAX,RDX
                {
                    let v = r[0] | r[2];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb642  49c1e813  SHR R8,0x13
                {
                    let v = r[8];
                    let n = 19u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb646  250000001c  AND EAX,0x1c000000
                {
                    let v = (r[0] & 0xffffffffu64 & 0x1c000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb64b  48ba000000001c000000  MOV RDX,0x1c00000000
                {
                    r[2] = 0x1c00000000u64;
                }
                // 1400cb655  4c0bc0  OR R8,RAX
                {
                    let v = r[8] | r[0];
                    f.logic(v, 64);
                    r[8] = v;
                }
                // 1400cb658  48b80000008003000000  MOV RAX,0x380000000
                {
                    r[0] = 0x380000000u64;
                }
                // 1400cb662  4c23c8  AND R9,RAX
                {
                    let v = r[9] & r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb665  49c1e814  SHR R8,0x14
                {
                    let v = r[8];
                    let n = 20u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[8] = v;
                    }
                }
                // 1400cb669  498bc2  MOV RAX,R10
                {
                    r[0] = r[10];
                }
                // 1400cb66c  4181e20000001c  AND R10D,0x1c000000
                {
                    let v = (r[10] & 0xffffffffu64 & 0x1c000000u64) & 0xffffffffu64;
                    f.logic(v, 32);
                    r[10] = (v) & 0xffff_ffff;
                }
                // 1400cb673  4823c2  AND RAX,RDX
                {
                    let v = r[0] & r[2];
                    f.logic(v, 64);
                    r[0] = v;
                }
                // 1400cb676  4c0bc8  OR R9,RAX
                {
                    let v = r[9] | r[0];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb679  49c1e90b  SHR R9,0xb
                {
                    let v = r[9];
                    let n = 11u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                // 1400cb67d  4d0bca  OR R9,R10
                {
                    let v = r[9] | r[10];
                    f.logic(v, 64);
                    r[9] = v;
                }
                // 1400cb680  49c1e914  SHR R9,0x14
                {
                    let v = r[9];
                    let n = 20u32;
                    if n != 0 {
                        let v = v >> n;
                        f.logic(v, 64);
                        r[9] = v;
                    }
                }
                pc = 0x1400cb684;
            }
            0x1400cb684 => {
                // 1400cb684  f605f55a2f000c  TEST byte ptr [0x1403c1180],0xc
                {
                    let v = (m.read::<8>(0u64.wrapping_add(0x1403c1180u64)) & 0xcu64) & 0xffu64;
                    f.logic(v, 8);
                }
                // 1400cb68b  743e  JZ 0x1400cb6cb
                pc = if f.z { 0x1400cb6cb } else { 0x1400cb68d };
                continue;
            }
            0x1400cb68d => {
                // 1400cb68d  438b848ee0121000  MOV EAX,dword ptr [R14 + R9*0x4 + 0x1012e0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[9].wrapping_mul(0x4u64))
                            .wrapping_add(0x1012e0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb695  43038486e01a1000  ADD EAX,dword ptr [R14 + R8*0x4 + 0x101ae0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[8].wrapping_mul(0x4u64))
                                .wrapping_add(0x101ae0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb69d  99  CDQ
                {
                    r[2] = ((r[0] as i32) >> 31) as u32 as u64;
                }
                // 1400cb69e  2bc2  SUB EAX,EDX
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, r[2] & 0xffffffffu64, 32, true);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb6a0  d1f8  SAR EAX,0x1
                {
                    let v = r[0] & 0xffffffffu64;
                    let n = 1u32;
                    if n != 0 {
                        let v = (((v as i32) >> n) as u64) & 0xffffffffu64;
                        f.logic(v, 32);
                        r[0] = (v) & 0xffff_ffff;
                    }
                }
                // 1400cb6a2  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb6a6  4c8525635a2f00  TEST qword ptr [0x1403c1110],R12
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[12];
                    f.logic(v, 64);
                }
                // 1400cb6ad  7509  JNZ 0x1400cb6b8
                pc = if !f.z { 0x1400cb6b8 } else { 0x1400cb6af };
                continue;
            }
            0x1400cb6af => {
                // 1400cb6af  0500002300  ADD EAX,0x230000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x230000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb6b4  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                pc = 0x1400cb6b8;
            }
            0x1400cb6b8 => {
                // 1400cb6b8  4c853d515a2f00  TEST qword ptr [0x1403c1110],R15
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[15];
                    f.logic(v, 64);
                }
                // 1400cb6bf  7541  JNZ 0x1400cb702
                pc = if !f.z { 0x1400cb702 } else { 0x1400cb6c1 };
                continue;
            }
            0x1400cb6c1 => {
                // 1400cb6c1  4181433800002300  ADD dword ptr [R11 + 0x38],0x230000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x230000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                // 1400cb6c9  eb37  JMP 0x1400cb702
                pc = 0x1400cb702;
                continue;
            }
            0x1400cb6cb => {
                // 1400cb6cb  438b8486e01a1000  MOV EAX,dword ptr [R14 + R8*0x4 + 0x101ae0]
                {
                    r[0] = (m.read::<32>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[8].wrapping_mul(0x4u64))
                            .wrapping_add(0x101ae0u64),
                    )) & 0xffff_ffff;
                }
                // 1400cb6d3  4303848ee0121000  ADD EAX,dword ptr [R14 + R9*0x4 + 0x1012e0]
                {
                    let v = f.arithmetic(
                        r[0] & 0xffffffffu64,
                        m.read::<32>(
                            0u64.wrapping_add(r[14])
                                .wrapping_add(r[9].wrapping_mul(0x4u64))
                                .wrapping_add(0x1012e0u64),
                        ),
                        32,
                        false,
                    );
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb6db  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                // 1400cb6df  4c85252a5a2f00  TEST qword ptr [0x1403c1110],R12
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[12];
                    f.logic(v, 64);
                }
                // 1400cb6e6  7509  JNZ 0x1400cb6f1
                pc = if !f.z { 0x1400cb6f1 } else { 0x1400cb6e8 };
                continue;
            }
            0x1400cb6e8 => {
                // 1400cb6e8  0500004600  ADD EAX,0x460000
                {
                    let v = f.arithmetic(r[0] & 0xffffffffu64, 0x460000u64, 32, false);
                    r[0] = (v) & 0xffff_ffff;
                }
                // 1400cb6ed  41894338  MOV dword ptr [R11 + 0x38],EAX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x38u64),
                        r[0] & 0xffffffffu64,
                    );
                }
                pc = 0x1400cb6f1;
            }
            0x1400cb6f1 => {
                // 1400cb6f1  4c853d185a2f00  TEST qword ptr [0x1403c1110],R15
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[15];
                    f.logic(v, 64);
                }
                // 1400cb6f8  7508  JNZ 0x1400cb702
                pc = if !f.z { 0x1400cb702 } else { 0x1400cb6fa };
                continue;
            }
            0x1400cb6fa => {
                // 1400cb6fa  4181433800004600  ADD dword ptr [R11 + 0x38],0x460000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x460000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                pc = 0x1400cb702;
            }
            0x1400cb702 => {
                // 1400cb702  486305675a2f00  MOVSXD RAX,dword ptr [0x1403c1170]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1170u64))) as i32 as i64) as u64;
                }
                // 1400cb709  4c8bbc2438010000  MOV R15,qword ptr [RSP + 0x138]
                {
                    r[15] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x138u64));
                }
                // 1400cb711  4c8bac2448010000  MOV R13,qword ptr [RSP + 0x148]
                {
                    r[13] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x148u64));
                }
                // 1400cb719  4c8ba42450010000  MOV R12,qword ptr [RSP + 0x150]
                {
                    r[12] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x150u64));
                }
                // 1400cb721  498b94c6f0e20e00  MOV RDX,qword ptr [R14 + RAX*0x8 + 0xee2f0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xee2f0u64),
                    );
                }
                // 1400cb729  488515e0592f00  TEST qword ptr [0x1403c1110],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb730  488bbc2458010000  MOV RDI,qword ptr [RSP + 0x158]
                {
                    r[7] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x158u64));
                }
                // 1400cb738  488bb42460010000  MOV RSI,qword ptr [RSP + 0x160]
                {
                    r[6] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x160u64));
                }
                // 1400cb740  488b9c2468010000  MOV RBX,qword ptr [RSP + 0x168]
                {
                    r[3] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x168u64));
                }
                // 1400cb748  751d  JNZ 0x1400cb767
                pc = if !f.z { 0x1400cb767 } else { 0x1400cb74a };
                continue;
            }
            0x1400cb74a => {
                // 1400cb74a  488515c7592f00  TEST qword ptr [0x1403c1118],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1118u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb751  750a  JNZ 0x1400cb75d
                pc = if !f.z { 0x1400cb75d } else { 0x1400cb753 };
                continue;
            }
            0x1400cb753 => {
                // 1400cb753  4181433400002800  ADD dword ptr [R11 + 0x34],0x280000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x280000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                // 1400cb75b  eb1b  JMP 0x1400cb778
                pc = 0x1400cb778;
                continue;
            }
            0x1400cb75d => {
                // 1400cb75d  4181433400001400  ADD dword ptr [R11 + 0x34],0x140000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0x140000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                // 1400cb765  eb11  JMP 0x1400cb778
                pc = 0x1400cb778;
                continue;
            }
            0x1400cb767 => {
                // 1400cb767  488515aa592f00  TEST qword ptr [0x1403c1118],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1118u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb76e  7508  JNZ 0x1400cb778
                pc = if !f.z { 0x1400cb778 } else { 0x1400cb770 };
                continue;
            }
            0x1400cb770 => {
                // 1400cb770  4181433400000a00  ADD dword ptr [R11 + 0x34],0xa0000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64)),
                        0xa0000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x34u64), v);
                }
                pc = 0x1400cb778;
            }
            0x1400cb778 => {
                // 1400cb778  486305f5592f00  MOVSXD RAX,dword ptr [0x1403c1174]
                {
                    r[0] = ((m.read::<32>(0u64.wrapping_add(0x1403c1174u64))) as i32 as i64) as u64;
                }
                // 1400cb77f  498b94c6f0e20e00  MOV RDX,qword ptr [R14 + RAX*0x8 + 0xee2f0]
                {
                    r[2] = m.read::<64>(
                        0u64.wrapping_add(r[14])
                            .wrapping_add(r[0].wrapping_mul(0x8u64))
                            .wrapping_add(0xee2f0u64),
                    );
                }
                // 1400cb787  4885158a592f00  TEST qword ptr [0x1403c1118],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1118u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb78e  4c8bb42440010000  MOV R14,qword ptr [RSP + 0x140]
                {
                    r[14] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x140u64));
                }
                // 1400cb796  751d  JNZ 0x1400cb7b5
                pc = if !f.z { 0x1400cb7b5 } else { 0x1400cb798 };
                continue;
            }
            0x1400cb798 => {
                // 1400cb798  48851571592f00  TEST qword ptr [0x1403c1110],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb79f  750a  JNZ 0x1400cb7ab
                pc = if !f.z { 0x1400cb7ab } else { 0x1400cb7a1 };
                continue;
            }
            0x1400cb7a1 => {
                // 1400cb7a1  4181433800002800  ADD dword ptr [R11 + 0x38],0x280000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x280000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                // 1400cb7a9  eb1b  JMP 0x1400cb7c6
                pc = 0x1400cb7c6;
                continue;
            }
            0x1400cb7ab => {
                // 1400cb7ab  4181433800001400  ADD dword ptr [R11 + 0x38],0x140000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0x140000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                // 1400cb7b3  eb11  JMP 0x1400cb7c6
                pc = 0x1400cb7c6;
                continue;
            }
            0x1400cb7b5 => {
                // 1400cb7b5  48851554592f00  TEST qword ptr [0x1403c1110],RDX
                {
                    let v = m.read::<64>(0u64.wrapping_add(0x1403c1110u64)) & r[2];
                    f.logic(v, 64);
                }
                // 1400cb7bc  7508  JNZ 0x1400cb7c6
                pc = if !f.z { 0x1400cb7c6 } else { 0x1400cb7be };
                continue;
            }
            0x1400cb7be => {
                // 1400cb7be  4181433800000a00  ADD dword ptr [R11 + 0x38],0xa0000
                {
                    let v = f.arithmetic(
                        m.read::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64)),
                        0xa0000u64,
                        32,
                        false,
                    );
                    m.write::<32>(0u64.wrapping_add(r[11]).wrapping_add(0x38u64), v);
                }
                pc = 0x1400cb7c6;
            }
            0x1400cb7c6 => {
                // 1400cb7c6  488b442428  MOV RAX,qword ptr [RSP + 0x28]
                {
                    r[0] = m.read::<64>(0u64.wrapping_add(r[4]).wrapping_add(0x28u64));
                }
                // 1400cb7cb  49894318  MOV qword ptr [R11 + 0x18],RAX
                {
                    m.write::<64>(0u64.wrapping_add(r[11]).wrapping_add(0x18u64), r[0]);
                }
                // 1400cb7cf  498bc3  MOV RAX,R11
                {
                    r[0] = r[11];
                }
                // 1400cb7d2  41894b30  MOV dword ptr [R11 + 0x30],ECX
                {
                    m.write::<32>(
                        0u64.wrapping_add(r[11]).wrapping_add(0x30u64),
                        r[1] & 0xffffffffu64,
                    );
                }
                // 1400cb7d6  4881c470010000  ADD RSP,0x170
                {
                    let v = f.arithmetic(r[4], 0x170u64, 64, false);
                    r[4] = v;
                }
                // 1400cb7dd  5d  POP RBP
                {
                    r[5] = m.read::<64>(r[4]);
                    r[4] = r[4].wrapping_add(8);
                }
                // 1400cb7de  c3  RET
                *registers = r;
                *simd = x;
                return r[0];
            }
            _ => unreachable!("unregistered original basic block {pc:x}"),
        }
    }
}
