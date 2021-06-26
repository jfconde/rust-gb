use super::REG_A;
use super::REG_B;
use super::REG_C;
use super::REG_D;
use super::REG_E;
use super::REG_F;
use super::REG_H;
use super::REG_L;

fn cpu_from_data(data: &mut Vec<u8>) -> super::CPU {
    let mut cpu = super::CPU::new();
    data.resize(1024, 0);
    cpu.reg_pc = 0x0;
    cpu.read_rom(&data);
    return cpu;
}

#[test]
fn test_pc_advances_correctly() {
    let mut cpu = cpu_from_data(&mut vec![0x02, 0x02, 0x06, 0x99]);
    cpu.regs[REG_A.1] = 0x23;
    cpu.regs[REG_B.1] = 0x10;
    cpu.set_bc(0xFF80);
    cpu.exec_inst(false);
    cpu.regs[REG_A.1] = 0x46;
    cpu.set_bc(0xFF81);
    cpu.exec_inst(false);
    cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(0xFF80), 0x23);
    assert_eq!(cpu.mmu.rb(0xFF81), 0x46);
    assert_eq!(cpu.regs[REG_B.1], 0x99);
}

#[test]
fn test_cpu_opcode_0x01() {
    let val_hi = 0x12;
    let val_lo = 0x3F;
    let mut cpu = cpu_from_data(&mut vec![0x01, val_lo, val_hi]);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], val_hi);
    assert_eq!(cpu.regs[REG_C.1], val_lo);
    assert_eq!(nops, 3);
}

#[test]
fn test_cpu_opcode_0x02() {
    let mut cpu = cpu_from_data(&mut vec![0x02]);
    cpu.regs[REG_A.1] = 0x23;
    // Load address 0xFF80 into BC
    cpu.set_bc(0xFF80);
    // Now write into the specified address the value of A.
    let nops = cpu.exec_inst(false);
    // Read back the value
    assert_eq!(cpu.mmu.rb(0xFF80), 0x23);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x06() {
    // Load 0x99 into B
    let mut cpu = cpu_from_data(&mut vec![0x06, 0x99]);
    // Set B to have random data
    cpu.regs[REG_B.1] = 0x10;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], 0x99);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x0A() {
    let mut cpu = cpu_from_data(&mut vec![0x0A]);
    let addr = 0xFF80;
    let value = 0x77;
    cpu.set_bc(addr);
    // Write the value in the address we just put in bc
    cpu.mmu.wb(addr, value);
    let nops = cpu.exec_inst(false);
    // Expect register A to have this value
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x0E() {
    let value = 0x99;
    let mut cpu = cpu_from_data(&mut vec![0x0E, value]);
    cpu.regs[REG_C.1] = 0x12;
    let nops = cpu.exec_inst(false);
    // Expect register A to have this value
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x11() {
    let val_hi = 0x12;
    let val_lo = 0x3F;
    let mut cpu = cpu_from_data(&mut vec![0x11, val_lo, val_hi]);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], val_hi);
    assert_eq!(cpu.regs[REG_E.1], val_lo);
    assert_eq!(nops, 3);
}

#[test]
fn test_cpu_opcode_0x12() {
    let mut cpu = cpu_from_data(&mut vec![0x12]);
    let value = 0x13;
    let addr = 0xFF80;
    cpu.regs[REG_A.1] = value;
    cpu.set_de(addr);
    let nops = cpu.exec_inst(false);
    // Expect the location of address written in DE to have the value.
    assert_eq!(cpu.mmu.rb(addr), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x16() {
    let value = 0x13;
    let mut cpu = cpu_from_data(&mut vec![0x16, value]);
    cpu.regs[REG_D.1] = 0x77;
    let nops = cpu.exec_inst(false);
    // Expect the location of address written in DE to have the value.
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x1A() {
    let value = 0x13;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0x1A]);
    cpu.set_de(addr); // Set DE (address)
    cpu.mmu.wb(addr, value); // Set value in memory
    cpu.regs[REG_A.1] = 0x11; // Trash data
    let nops = cpu.exec_inst(false);
    // Expect A to contain the contents of location DE in memory
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x1E() {
    let value = 0x88;
    let mut cpu = cpu_from_data(&mut vec![0x1E, value]);
    cpu.regs[REG_E.1] = 0x77;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x21() {
    let val_hi = 0x12;
    let val_lo = 0x3F;
    let mut cpu = cpu_from_data(&mut vec![0x21, val_lo, val_hi]);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], val_hi);
    assert_eq!(cpu.regs[REG_L.1], val_lo);
    assert_eq!(nops, 3);
}

#[test]
fn test_cpu_opcode_0x22() {
    let value = 0x88;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0x22]);
    cpu.mmu.wb(addr, 0x00);
    cpu.regs[REG_A.1] = value;
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(addr), value);
    assert_eq!(cpu.get_hl(), addr + 1);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x26() {
    let value = 0x55;
    let mut cpu = cpu_from_data(&mut vec![0x26, value]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x2A() {
    let value = 0x55;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0x2A]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(cpu.get_hl(), addr + 1);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x2E() {
    let value = 0x97;
    let mut cpu = cpu_from_data(&mut vec![0x2E, value]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x31() {
    let val_hi = 0x12;
    let val_lo = 0x3F;
    let mut cpu = cpu_from_data(&mut vec![0x31, val_lo, val_hi]);
    let nops = cpu.exec_inst(false);
    let value = (val_hi as u16) << 8 | val_lo as u16; 
    assert_eq!(cpu.reg_sp, value);
    assert_eq!(nops, 3);
}

#[test]
fn test_cpu_opcode_0x32() {
    let value = 0x55;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0x32]);
    cpu.mmu.wb(addr, 0x10); // Trash data
    cpu.regs[REG_A.1] = value;
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(addr), value);
    assert_eq!(cpu.get_hl(), addr - 1);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x36() {
    let value = 0x13;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0x36, value]);
    cpu.mmu.wb(addr, 0x10); // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(addr), value);
    assert_eq!(nops, 3);
}

#[test]
fn test_cpu_opcode_0x3A() {
    let value = 0x71;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0x3A]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.mmu.wb(addr, value);
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(cpu.get_hl(), addr - 1);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x3E() {
    let value = 0x97;
    let mut cpu = cpu_from_data(&mut vec![0x3E, value]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x40() {
    let value = 0x97;
    let mut cpu = cpu_from_data(&mut vec![0x40]);
    cpu.regs[REG_B.1] = value; // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x41() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x41]);
    cpu.regs[REG_B.1] = 0x10; // Trash data
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x42() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x42]);
    cpu.regs[REG_B.1] = 0x10; // Trash data
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x43() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x43]);
    cpu.regs[REG_B.1] = 0x10; // Trash data
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x44() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x44]);
    cpu.regs[REG_B.1] = 0x10; // Trash data
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x45() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x45]);
    cpu.regs[REG_B.1] = 0x10; // Trash data
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x46() {
    let value = 0x93;
    let addr = 0xff80;
    let mut cpu = cpu_from_data(&mut vec![0x46]);
    cpu.set_hl(addr);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_B.1] = 0x10; // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x47() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x47]);
    cpu.regs[REG_B.1] = 0x10; // Trash data
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_B.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x48() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x48]);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.regs[REG_B.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x49() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x49]);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x4A() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x4A]);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x4B() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x4B]);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x4C() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x4C]);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x4D() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x4D]);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x4E() {
    let addr = 0xFF80;
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x4E]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x4F() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x4F]);
    cpu.regs[REG_C.1] = 0x10; // Trash data
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_C.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x50() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x50]);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.regs[REG_B.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x51() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x51]);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x52() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x52]);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x53() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x53]);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x54() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x54]);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x55() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x55]);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x56() {
    let addr = 0xFF80;
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x56]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x57() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x57]);
    cpu.regs[REG_D.1] = 0x10; // Trash data
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_D.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x58() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x58]);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.regs[REG_B.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x59() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x59]);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x5A() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x5A]);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x5B() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x5B]);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x5C() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x5C]);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x5D() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x5D]);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x5E() {
    let addr = 0xFF80;
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x5E]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x5F() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x5F]);
    cpu.regs[REG_E.1] = 0x10; // Trash data
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_E.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x60() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x60]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.regs[REG_B.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x61() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x61]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x62() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x62]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x63() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x63]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x64() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x64]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x65() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x65]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x66() {
    let addr = 0xFF80;
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x66]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x67() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x67]);
    cpu.regs[REG_H.1] = 0x10; // Trash data
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_H.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x68() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x68]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.regs[REG_B.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x69() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x69]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x6A() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x6A]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x6B() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x6B]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x6C() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x6C]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x6D() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x6D]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x6E() {
    let addr = 0xFF80;
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x6E]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x6F() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x6F]);
    cpu.regs[REG_L.1] = 0x10; // Trash data
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_L.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x70() {
    let value = 0x93;
    let addr = 0xFF90;
    let mut cpu = cpu_from_data(&mut vec![0x70]);
    cpu.set_hl(addr);
    cpu.regs[REG_B.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(cpu.get_hl()), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x71() {
    let value = 0x93;
    let addr = 0xFF90;
    let mut cpu = cpu_from_data(&mut vec![0x71]);
    cpu.set_hl(addr);
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(cpu.get_hl()), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x72() {
    let value = 0x93;
    let addr = 0xFF90;
    let mut cpu = cpu_from_data(&mut vec![0x72]);
    cpu.set_hl(addr);
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(cpu.get_hl()), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x73() {
    let value = 0x93;
    let addr = 0xFF90;
    let mut cpu = cpu_from_data(&mut vec![0x73]);
    cpu.set_hl(addr);
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(cpu.get_hl()), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x74() {
    let value = 0x93;
    let addr = 0xFF90;
    let mut cpu = cpu_from_data(&mut vec![0x74]);
    cpu.set_hl(addr);
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(cpu.get_hl()), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x75() {
    let value = 0x93;
    let addr = 0xFF90;
    let mut cpu = cpu_from_data(&mut vec![0x75]);
    cpu.set_hl(addr);
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(cpu.get_hl()), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x77() {
    let value = 0x93;
    let addr = 0xFF90;
    let mut cpu = cpu_from_data(&mut vec![0x77]);
    cpu.set_hl(addr);
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(cpu.get_hl()), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x78() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x78]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.regs[REG_B.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x79() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x79]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.regs[REG_C.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x7A() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x7A]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.regs[REG_D.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x7B() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x7B]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.regs[REG_E.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x7C() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x7C]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.regs[REG_H.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x7D() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x7D]);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.regs[REG_L.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0x7E() {
    let addr = 0xFF80;
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x7E]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_A.1] = 0x10; // Trash data
    cpu.set_hl(addr);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0x7F() {
    let value = 0x93;
    let mut cpu = cpu_from_data(&mut vec![0x7F]);
    cpu.regs[REG_A.1] = value;
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 1);
}

#[test]
fn test_cpu_opcode_0xE0() {
    let offset = 0x90;
    let value = 0x65;
    let mut cpu = cpu_from_data(&mut vec![0xE0, offset]);
    cpu.regs[REG_A.1] = value;
    cpu.mmu.wb(0xff00 + offset as u16, 0x01); // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(0xff00 + offset as u16), value);
    assert_eq!(nops, 3);
}

#[test]
fn test_cpu_opcode_0xE2() {
    let offset = 0x90;
    let value = 0x65;
    let mut cpu = cpu_from_data(&mut vec![0xE2]);
    cpu.regs[REG_A.1] = value;
    cpu.regs[REG_C.1] = offset;
    cpu.mmu.wb(0xff00 + offset as u16, 0x01); // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(0xff00 + offset as u16), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0xEA() {
    let value = 0x65;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0xEA, 0x80, 0xFF]);
    cpu.regs[REG_A.1] = value;
    cpu.mmu.wb(addr, 0x01); // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.mmu.rb(addr), value);
    assert_eq!(nops, 4);
}

#[test]
fn test_cpu_opcode_0xF0() {
    let offset = 0x80;
    let addr = 0xFF80;
    let value = 0x7D;
    let mut cpu = cpu_from_data(&mut vec![0xF0, offset]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_A.1] = 0x01; // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 3);
}


#[test]
fn test_cpu_opcode_0xF2() {
    let offset = 0x90;
    let value = 0x65;
    let mut cpu = cpu_from_data(&mut vec![0xF2]);
    cpu.regs[REG_A.1] = 0x33; // Trash data
    cpu.regs[REG_C.1] = offset;
    cpu.mmu.wb(0xff00 + offset as u16, value);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0xF8() {
    let value = 0x2345;
    let mut cpu = cpu_from_data(&mut vec![0xF8]);
    cpu.reg_sp = 0x00; // Trash data
    cpu.set_hl(value);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.get_hl(), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0xF9() {
    let value = 0x2345;
    let mut cpu = cpu_from_data(&mut vec![0xF9]);
    cpu.reg_sp = 0x00; // Trash data
    cpu.set_hl(value);
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.get_hl(), value);
    assert_eq!(nops, 2);
}

#[test]
fn test_cpu_opcode_0xFA() {
    let value = 0x29;
    let addr = 0xFF80;
    let mut cpu = cpu_from_data(&mut vec![0xFA, 0x80, 0xFF]);
    cpu.mmu.wb(addr, value);
    cpu.regs[REG_A.1] = 0x01; // Trash data
    let nops = cpu.exec_inst(false);
    assert_eq!(cpu.regs[REG_A.1], value);
    assert_eq!(nops, 4);
}
