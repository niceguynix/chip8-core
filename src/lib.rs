pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;

const NUM_KEYS: usize = 16;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

const START_ADDR: u16 = 0x200;

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
}

impl Emu {
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}

impl Emu {
    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0xF00) >> 8;
        let digit3 = (op & 0xF0) >> 4;
        let digit4 = op & 0xF;

        println!(
            "Opcode- {:x} {:x} {:x} {:x}",
            digit1, digit2, digit3, digit4
        );

        match (digit1, digit2, digit3, digit4) {
            (0xF, _, 6, 5) => Self::load_reg(self, digit2),
            (0xF, _, 5, 5) => Self::store_reg(self, digit2),
            (0xF, _, 3, 3) => Self::convert_bcd(self, digit2),
            (0xF, _, 2, 9) => Self::set_font_address(self, digit2),
            (0xF, _, 1, 0xE) => Self::update_i_reg(self, digit2),
            (0xF, _, 1, 8) => Self::update_st(self, digit2),
            (0xF, _, 1, 5) => Self::update_dt(self, digit2),
            (0xF, _, 0, 0xA) => Self::wait_for_key_press(self, digit2),
            (0xF, _, 0, 7) => Self::copy_dt(self, digit2),
            (0xE, _, 0xA, 1) => Self::skip_if_key_not_pressed(self, digit2),
            (0xE, _, 9, 0xE) => Self::skip_on_key_press(self, digit2),
            (0xD, _, _, _) => Self::draw_sprite(self, digit2, digit3, digit4),
            (0xC, _, _, _) => Self::random_with_and(self, op, digit2),
            (0xB, _, _, _) => Self::jump_with_V(self, op),
            (0xA, _, _, _) => Self::set_i_reg(self, op),
            (9, _, _, 0) => Self::skip_if_not_eq(self, digit2, digit3),
            (8, _, _, 0xE) => Self::left_shift(self, digit2),
            (8, _, _, 7) => Self::reverse_sub_with_overflow(self, digit2, digit3),
            (8, _, _, 6) => Self::right_shift(self, digit2),
            (8, _, _, 5) => Self::sub_with_underflow(self, digit2, digit3),
            (8, _, _, 4) => Self::add_with_overdlow(self, digit2, digit3),
            (8, _, _, 3) => Self::xor(self, digit2, digit3),
            (8, _, _, 2) => Self::and(self, digit2, digit3),
            (8, _, _, 1) => Self::or(self, digit2, digit3),
            (8, _, _, 0) => Self::set_reg_from_reg(self, digit2, digit3),
            (7, _, _, _) => Self::add_and_set(self, op, digit2),
            (6, _, _, _) => Self::set_register(self, op, digit2),
            (5, _, _, 0) => Self::skip_if_2_reg(self, digit2, digit3),
            (4, _, _, _) => Self::skip_if_not(self, op, digit2),
            (3, _, _, _) => Self::skip_if(self, op, digit2),
            (2, _, _, _) => Self::call_subrt(self, op),
            (1, _, _, _) => Self::jump(self, op),
            (0, 0, 0xE, 0xE) => Self::ret_subrt(self),
            (0, 0, 0xE, 0) => Self::clr_scr(self),
            (0, 0, 0, 0) => (),
            (_, _, _, _) => unimplemented!(),
        }
    }
}

impl Emu {
    //clears the screen
    //Opcode-00E0
    fn clr_scr(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    //return from the subroutine
    //Opcode-ooE0
    fn ret_subrt(&mut self) {
        let ret_addr = self.pop();
        self.pc = ret_addr;
    }

    //jump instruction
    fn jump(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.pc = nnn;
    }

    //call subroutine
    //Opcode-2NNN
    fn call_subrt(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.push(self.pc);
        self.pc = nnn;
    }

    //Skip next if Vx==NN
    //Opcode-3XNN
    fn skip_if(&mut self, op: u16, x: u16) {
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x as usize] == nn {
            self.pc += 2;
        }
    }

    //Skip next if Vx!=NN
    //Opcode-4XNN
    fn skip_if_not(&mut self, op: u16, x: u16) {
        let nn = (op & 0xFF) as u8;
        if self.v_reg[x as usize] != nn {
            self.pc += 2;
        }
    }

    //Skip next if Vx==Vy
    //Opcode-5XY0
    fn skip_if_2_reg(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        if self.v_reg[x] == self.v_reg[y] {
            self.pc += 2;
        }
    }

    //Set register to nn
    //Opcode- 6XNN - VX = NN
    fn set_register(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        self.v_reg[x] = nn;
    }

    //Add and store to register
    //Opcode-7XNN - VX += NN
    fn add_and_set(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
    }

    //Set register from another register
    //Opcode- 8XY0 - VX = VY
    fn set_reg_from_reg(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] = self.v_reg[y];
    }

    //Or operation
    //Opcode-8XY1
    fn or(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] |= self.v_reg[y];
    }

    //And operation
    //Opcode-8XY2
    fn and(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] &= self.v_reg[y];
    }

    //Xor operation
    //Opcode-8XY3
    fn xor(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.v_reg[x] ^= self.v_reg[y];
    }

    //Addition with overflow
    //Opcode-8XY4 - VX += VY
    fn add_with_overdlow(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;

        let (result, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
        self.v_reg[x] = result;
        self.v_reg[0xF] = carry as u8;
    }

    //Subtraction with underflow
    //Opcode-8XY5 - VX -= VY
    fn sub_with_underflow(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;

        let (result, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

        self.v_reg[x] = result;
        self.v_reg[0xF] = !borrow as u8;
    }

    //Right shift
    //Opcode-8XY6 - VX »= 1
    fn right_shift(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let lsb = self.v_reg[x] & 1;
        self.v_reg[x] >>= 1;
        self.v_reg[0xF] = lsb;
    }

    //Subtraction with the registers in reverse
    //Opcode-8XY7 - VX = VY - VX
    fn reverse_sub_with_overflow(&mut self, digit2: u16, digit3: u16) {
        let y = digit2 as usize;
        let x = digit3 as usize;

        let (result, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
        //Storing in wrong register
        self.v_reg[y] = result;
        self.v_reg[0xF] = !borrow as u8;
    }

    //left shift with overflowed bit in the vf register
    //Opcode-8XYE - VX «= 1
    fn left_shift(&mut self, digit2: u16) {
        let x = digit2 as usize;
        //Check if equivalent
        let msb = self.v_reg[x] & 8;
        self.v_reg[x] <<= 1;
        self.v_reg[0xF] = msb;
    }

    //skip if not equal
    //Opcode-9XY0 - Skip if VX != VY
    fn skip_if_not_eq(&mut self, digit2: u16, digit3: u16) {
        let x = digit2 as usize;
        let y = digit3 as usize;

        if self.v_reg[x] != self.v_reg[y] {
            self.pc += 2;
        }
    }

    //Update I register
    //Opcode-ANNN - I = NNN
    fn set_i_reg(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.i_reg = nnn;
    }

    //Jump to address specified by Vo+opcode
    //Opcode-BNNN - Jump to V0 + NNN
    fn jump_with_V(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.pc = self.v_reg[0] as u16 + nnn;
    }

    //Opcode to generate random number
    // CXNN - VX = rand() & NN
    fn random_with_and(&mut self, op: u16, digit2: u16) {
        let x = digit2 as usize;
        let nn = (op & 0xFF) as u8;
        let rng: u8 = rand::random();
        self.v_reg[x] = rng & nn;
    }

    //Draw sprite
    //Opcode-DXYN - Draw Sprite
    fn draw_sprite(&mut self, digit2: u16, digit3: u16, digit4: u16) {
        let x_coord = self.v_reg[digit2 as usize] as u16;
        let y_coord = self.v_reg[digit3 as usize] as u16;

        let num_of_rows = digit4;
        let mut flipped = false;

        for y in 0..num_of_rows {
            let addr = self.i_reg + y;
            let pixels = self.ram[addr as usize];

            for x in 0..8 {
                if (pixels & (0b1000_0000 >> x)) != 0 {
                    let x = (x_coord + x) as usize % SCREEN_WIDTH;
                    let y = (y_coord + y) as usize % SCREEN_HEIGHT;

                    let idx = x + y * SCREEN_WIDTH;
                    //check if screen pixel flipped
                    flipped |= self.screen[idx];
                    self.screen[idx] ^= true;
                }
            }
        }
        //need to check
        // self.v_reg[0xF] = if flipped { 1 } else { self.v_reg[0xF] };
        self.v_reg[0xF] = if flipped { 1 } else { 0 };
    }

    //Skips next instruction if specified key is pressed
    //Opcode-EX9E - Skip if Key Pressed
    fn skip_on_key_press(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let vx = self.v_reg[x];
        let key = self.keys[vx as usize];
        if key {
            self.pc += 2;
        }
    }

    //Skips next instruction if specified key is not pressed
    //Opcode-EXA1 - Skip if Key Not Pressed
    fn skip_if_key_not_pressed(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let vx = self.v_reg[x];
        let key = self.keys[vx as usize];
        if !key {
            self.pc += 2;
        }
    }

    //Copies Delay timer value into registers
    //Opcode-FX07 - VX = DT
    fn copy_dt(&mut self, digit2: u16) {
        let x = digit2 as usize;
        self.v_reg[x] = self.dt;
    }

    //Wait till a key is pressed
    //Opcode-FX0A - Wait for Key Press
    fn wait_for_key_press(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[x] = i as u8;
                pressed = true;
                break;
            }
        }
        //Repoint to current address
        if !pressed {
            self.pc -= 2;
        }
    }

    //Update delay timer
    //Opcode-FX15 - DT = VX
    fn update_dt(&mut self, digit2: u16) {
        let x = digit2 as usize;
        self.dt = self.v_reg[x];
    }

    //Update sound timer
    //Opcode-FX18 - ST = VX
    fn update_st(&mut self, digit2: u16) {
        let x = digit2 as usize;
        self.st = self.v_reg[x];
    }

    //Update i register from a v register
    //Opcode-FX1E - I += VX
    fn update_i_reg(&mut self, digit2: u16) {
        let x = digit2 as usize;
        self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
    }

    //Set i register to font address
    //Opcode-FX29
    fn set_font_address(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let c = self.v_reg[x] as u16;
        self.i_reg = c * 5;
    }

    //Converts number from v register to bcd
    //Opcode-FX33
    fn convert_bcd(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let vx = self.v_reg[x] as f64;

        let hundreds = (vx / 100.0).floor() as u8;
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        let ones = (vx % 10.0) as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }

    //Store register into ram
    //Opcode-FX55
    fn store_reg(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let i = self.i_reg as usize;
        for idx in 0..x {
            self.ram[i + idx] = self.v_reg[idx];
        }
    }

    //Load register from ram
    //Opcode-FX65
    fn load_reg(&mut self, digit2: u16) {
        let x = digit2 as usize;
        let i = self.i_reg as usize;
        for idx in 0..x {
            self.v_reg[idx] = self.ram[i + idx];
        }
    }
}
