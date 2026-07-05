use crate::draw::{BLACK, RED, WHITE};
use crate::pnp;
use crate::pnp::{print, println};
use crate::utils::CircularCounter;

pub trait MaxDigits{
    type Type;
    const DIGITS: usize;
    fn from_digits(digits: &[CircularCounter]) -> Self::Type;
}

// 255
impl MaxDigits for u8 {
    type Type = u8;
    const DIGITS: usize = 3;

    fn from_digits(digits: &[CircularCounter]) -> Self::Type {
        // Use saturating mul and add, to avoid integer overflow
        digits.iter().fold(0, |acc, digit|acc.saturating_mul(10).saturating_add(digit.value() as u8))
    }
}
// 65,535
impl MaxDigits for u16 {
    type Type = u16;
    const DIGITS: usize = 5;
    fn from_digits(digits: &[CircularCounter]) -> Self::Type {
        // Use saturating mul and add, to avoid integer overflow
        digits.iter().fold(0, |acc, digit|acc.saturating_mul(10).saturating_add(digit.value() as u16))
    }
}
// 4,294,967,295
impl MaxDigits for u32 {
    type Type = u32;
    const DIGITS: usize = 10;

    fn from_digits(digits: &[CircularCounter]) -> Self::Type {
        // Use saturating mul and add, to avoid integer overflow
        digits.iter().fold(0, |acc, digit|acc.saturating_mul(10).saturating_add(digit.value() as u32))
    }
}
// 18,446,744,073,709,551,615 (not sure if actually compatible with platform, but may as well add support)
impl MaxDigits for u64 {
    type Type = u64;
    const DIGITS: usize = 20;
    fn from_digits(digits: &[CircularCounter]) -> Self::Type {
        // Use saturating mul and add, to avoid integer overflow
        digits.iter().fold(0, |acc, digit|acc.saturating_mul(10).saturating_add(digit.value() as u64))
    }
}
// Support for u128 would restrict too much the space for the label


// Low-overhead number input selector where the data stays in the stack rather than need heap allocation.
// Relies upon the unstable "generic_const_exprs" feature to generically set the array of digits' size
// Since we're using a nightly toolchain for the build anyway, may as well.
pub struct NumberInput<'a, T: MaxDigits>
where [(); T::DIGITS]:
{
    digits: [CircularCounter; T::DIGITS],
    cursor: CircularCounter,
    label: &'a str,
}

impl<'a,T: MaxDigits> NumberInput<'a, T>
where [(); T::DIGITS]: {
    pub fn new(label: &'a str) -> NumberInput<'a, T> {
        NumberInput{
            digits: [CircularCounter::new(0,9); T::DIGITS],
            cursor: CircularCounter::new(0, T::DIGITS - 1),
            label,
        }
    }

    pub fn set_value(&mut self, mut val: usize) {
        for i in (0..T::DIGITS).rev() {
            self.digits[i].set(val%10);
            val /= 10;
            if val == 0 {
                self.cursor.set(i-1);
                break;
            }
        }
    }
    pub fn value(&self) -> T::Type
    {
        T::from_digits(&self.digits)
    }

    pub fn update(&mut self) {
        // Handle input, first cursor movement, then digit changes
        if pnp::is_just_pressed(pnp::Button::Dright){
            self.cursor.increment();
        } else if pnp::is_just_pressed(pnp::Button::Dleft){
            self.cursor.decrement();
        } else if pnp::is_just_pressed(pnp::Button::Dup) {
            self.digits[self.cursor.value()].increment();
        } else if pnp::is_just_pressed(pnp::Button::Ddown) {
            self.digits[self.cursor.value()].decrement();
        }
    }

    pub fn draw_number(&self){
        let mut still_zero = true;
        let cursor_pos = self.cursor.value();
        // Draw all numbers before the cursor (drawing them black so long as they are leading 0s)
        for i in 0..cursor_pos {
            let value = self.digits[i].value();
            let color = if value == 0 && still_zero {
                BLACK
            } else {
                still_zero = false;
                WHITE
            };
            print!(color = color, "{}", value);
        }

        // Draw cursor number as red
        print!(color = RED, "{}", self.digits[self.cursor.value()].value());

        // Draw remaining numbers as white
        for i in cursor_pos+1..T::DIGITS {
            let value = self.digits[i].value();
            print!(color = WHITE, "{}", value);
        }
    }

    pub fn draw_line(&self){
        print!(color = WHITE, "{}: ", self.label);
        self.draw_number();
        println!();
    }
    pub fn draw_header(){
        println!("<-/-> to move cursor");
        println!("Up/Down to change num.");
    }

    pub fn draw(&self) {
        Self::draw_header();
        println!();
        self.draw_line();
    }
}