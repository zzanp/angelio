use std::{
    fs,
    iter::{Enumerate, Peekable},
    str::{Chars, FromStr},
};

use rppal::{
    gpio::{Gpio, Level, Pin},
    pwm::{Channel, Polarity, Pwm},
};

enum RegRet {
    Normal(u32),
    Floating(f32),
}

pub struct Angelio {
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,

    pub f1: f32,
    pub f2: f32,
    pub f3: f32,
    pub f4: f32,

    pid_p: f32,
    pid_i: f32,
    pid_d: f32,
    pid_setpoint: f32,
    pid_prev_error: f32,
    pid_tot_error: f32,
    code: String,
}

impl Angelio {
    pub fn new(path: &str) -> Angelio {
        let source =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to open file {path}"));
        Angelio::from_string(source)
    }

    pub fn from_string(source: String) -> Angelio {
        let mut s = source;
        s.push('\0');
        Angelio {
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            f1: 0.,
            f2: 0.,
            f3: 0.,
            f4: 0.,
            pid_p: 0.,
            pid_i: 0.,
            pid_d: 0.,
            pid_setpoint: 0.,
            pid_prev_error: 0.,
            pid_tot_error: 0.,
            code: s,
        }
    }

    pub fn from_str(source: &str) -> Angelio {
        Angelio::from_string(source.to_string())
    }

    fn get_number<T: FromStr>(
        &self,
        source: &mut Peekable<Enumerate<Chars>>,
        old_idx: usize,
    ) -> Result<T, <T as FromStr>::Err> {
        let mut val = String::new();
        while let Some((idx, c)) = source.peek() {
            if c.is_ascii_digit() || *c == '.' {
                val.push(*c);
                source.next();
            } else if val.is_empty() {
                panic!("Invalid value ({})", idx + 1);
            } else {
                let num = val.parse::<T>();
                return num;
            }
        }
        panic!("Can't get any number ({old_idx})",);
    }

    fn get_port(&self, port: u8) -> Pin {
        Gpio::new()
            .unwrap_or_else(|_| panic!("GPIO device could not be opened"))
            .get(port)
            .unwrap_or_else(|_| panic!("Could not get GPIO port {port}"))
    }

    fn get_register_argument(
        &self,
        source: &mut Peekable<Enumerate<Chars>>,
        old_idx: usize,
    ) -> String {
        let (ridx, regtype) = source
            .next()
            .unwrap_or_else(|| panic!("Unable to retrieve any register type ({})", old_idx + 1));
        if regtype != 'r' && regtype != 'f' {
            panic!("Invalid register type found ('{regtype}') ({})", ridx + 1);
        }

        let (nidx, regn) = source
            .next()
            .unwrap_or_else(|| panic!("No register number could be obtained ({})", old_idx + 2));
        let regni = regn.to_digit(10).unwrap_or_else(|| {
            panic!(
                "Specified register number is not a number ('{}') ({})",
                regn,
                old_idx + 3
            )
        });
        if !(1..=4).contains(&regni) {
            panic!("Invalid register number found ({regn}) ({})", nidx + 1);
        }

        let mut reg = String::new();
        reg.push(regtype);
        reg.push(regn);

        reg
    }

    fn get_register_value(&self, reg: String, old_idx: usize) -> Result<RegRet, String> {
        match reg.as_str() {
            "r1" => Ok(RegRet::Normal(self.r1)),
            "r2" => Ok(RegRet::Normal(self.r2)),
            "r3" => Ok(RegRet::Normal(self.r3)),
            "r4" => Ok(RegRet::Normal(self.r4)),
            "f1" => Ok(RegRet::Floating(self.f1)),
            "f2" => Ok(RegRet::Floating(self.f1)),
            "f3" => Ok(RegRet::Floating(self.f3)),
            "f4" => Ok(RegRet::Floating(self.f4)),
            _ => Err(format!("Invalid register name {}", old_idx + 1)),
        }
    }

    fn get_register_value_as_array(&self, reg1: String, reg2: String, old_idx: usize) -> [f32; 2] {
        let mut args: [f32; 2] = [0., 0.];

        match self.get_register_value(reg1, old_idx + 1).unwrap() {
            RegRet::Normal(val) => args[0] = val as f32,
            RegRet::Floating(val) => args[0] = val,
        }

        match self.get_register_value(reg2, old_idx + 3).unwrap() {
            RegRet::Normal(val) => args[1] = val as f32,
            RegRet::Floating(val) => args[1] = val,
        }

        args
    }

    pub fn set_register(&mut self, register: u32, value: u32) -> Result<(), String> {
        match register {
            1 => self.r1 = value,
            2 => self.r2 = value,
            3 => self.r3 = value,
            4 => self.r4 = value,
            _ => return Err(format!("Invalid register number: {register}")),
        };
        Ok(())
    }

    pub fn set_float_register(&mut self, register: u32, value: f32) -> Result<(), String> {
        match register {
            1 => self.f1 = value,
            2 => self.f2 = value,
            3 => self.f3 = value,
            4 => self.f4 = value,
            _ => return Err(format!("Invalid register number: {register}")),
        };
        Ok(())
    }

    pub fn set_register_by_name(&mut self, reg: String, value: u32) -> Result<(), String> {
        match reg.as_str() {
            "r1" => self.r1 = value,
            "r2" => self.r2 = value,
            "r3" => self.r3 = value,
            "r4" => self.r4 = value,
            _ => return Err(format!("Invalid register: {reg}")),
        }
        Ok(())
    }

    pub fn set_float_register_by_name(&mut self, reg: String, value: f32) -> Result<(), String> {
        match reg.as_str() {
            "f1" => self.f1 = value,
            "f2" => self.f2 = value,
            "f3" => self.f3 = value,
            "f4" => self.f4 = value,
            _ => return Err(format!("Invalid register: {reg}")),
        };
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mcode = &mut (self.code.clone());
        let mut source = mcode.chars().enumerate().peekable();
        while let Some((idx, c)) = source.next() {
            match c {
                'P' => {
                    let p = self
                        .get_number::<f32>(&mut source, idx)
                        .unwrap_or_else(|_| panic!("Value is not a valid float ({})", idx + 1));
                    self.pid_p = p;
                }
                'I' => {
                    let i = self
                        .get_number::<f32>(&mut source, idx)
                        .unwrap_or_else(|_| panic!("Value is not a valid float ({})", idx + 1));
                    self.pid_i = i
                }
                'D' => {
                    let d = self
                        .get_number::<f32>(&mut source, idx)
                        .unwrap_or_else(|_| panic!("Value is not a valid float ({})", idx + 1));
                    self.pid_d = d;
                }
                'q' => {
                    let setpoint = self
                        .get_number::<f32>(&mut source, idx)
                        .unwrap_or_else(|_| panic!("Value is not a valid float ({})", idx + 1));
                    self.pid_setpoint = setpoint;
                }
                'c' => {
                    let measurement = self
                        .get_number::<f32>(&mut source, idx)
                        .unwrap_or_else(|_| panic!("Value is not a valid float ({})", idx + 1));
                    let pos_error = self.pid_setpoint - measurement;
                    let verror = (pos_error - self.pid_prev_error) / 0.02;
                    self.pid_prev_error = pos_error;
                    if self.pid_i != 0.0 {
                        let a = self.pid_tot_error + pos_error * 0.02;
                        if a < -1.0 / self.pid_i {
                            self.pid_tot_error = -1.0 / self.pid_i;
                        } else if a > 1.0 / self.pid_i {
                            self.pid_tot_error = 1.0 / self.pid_i;
                        } else {
                            self.pid_tot_error = a;
                        }
                    }
                    let calculation = self.pid_p * pos_error + self.pid_i * self.pid_tot_error + self.pid_d * verror;
                    self.set_float_register(3, calculation)?;
                }
                'l' => {
                    let reg = self.get_register_argument(&mut source, idx);
                    let num = self
                        .get_number::<f32>(&mut source, idx)
                        .unwrap_or_else(|_| panic!("Value is not a valid number ({})", idx + 1));
                    if reg.starts_with('r') {
                        self.set_register_by_name(reg, num as u32)?;
                    } else {
                        self.set_float_register_by_name(reg, num)?;
                    }
                }
                '+' => {
                    let first_name = self.get_register_argument(&mut source, idx);
                    let second_name = self.get_register_argument(&mut source, idx);

                    let args = self.get_register_value_as_array(
                        first_name.to_owned(),
                        second_name.to_owned(),
                        idx,
                    );

                    let mut sum: f32 = 0.0;

                    for x in args {
                        sum += x;
                    }

                    if first_name.starts_with('r') && second_name.starts_with('r') {
                        self.r3 = sum as u32;
                    } else {
                        self.f3 = sum;
                    }
                }
                'T' => {
                    let first_name = self.get_register_argument(&mut source, idx);
                    let second_name = self.get_register_argument(&mut source, idx);
                    let args = self.get_register_value_as_array(
                        first_name.to_owned(),
                        second_name.to_owned(),
                        idx,
                    );

                    if first_name.starts_with('r') {
                        self.set_float_register_by_name(second_name.to_owned(), args[0])?;
                    } else {
                        self.set_register_by_name(second_name.to_owned(), args[0] as u32)?;
                    }

                    if second_name.starts_with('r') {
                        self.set_float_register_by_name(first_name, args[1])?;
                    } else {
                        self.set_register_by_name(first_name, args[1] as u32)?;
                    }
                }
                '!' => {
                    let reg = self.get_register_argument(&mut source, idx);
                    match self.get_register_value(reg, idx)? {
                        RegRet::Normal(val) => println!("{val}"),
                        RegRet::Floating(val) => println!("{val}"),
                    }
                }
                'o' => {
                    let reg = self.get_register_argument(&mut source, idx);
                    let port_number =
                        self.get_number::<u8>(&mut source, idx).unwrap_or_else(|_| {
                            panic!("Value is not a valid port number ({})", idx + 1)
                        });
                    let value = match self.get_register_value(reg, idx)? {
                        RegRet::Normal(val) => val,
                        RegRet::Floating(val) => val as u32,
                    };

                    let mut port = self.get_port(port_number).into_output();

                    match value {
                        0 => port.set_low(),
                        _ => port.set_high(),
                    }
                }
                'i' => {
                    let reg = self.get_register_argument(&mut source, idx);
                    let port_number =
                        self.get_number::<u8>(&mut source, idx).unwrap_or_else(|_| {
                            panic!("Value is not a valid port number ({})", idx + 1)
                        });
                    let port = self.get_port(port_number).into_input();

                    let out = match port.read() {
                        Level::High => 1,
                        Level::Low => 0,
                    };

                    if reg.starts_with('r') {
                        self.set_register_by_name(reg, out)?;
                    } else {
                        self.set_float_register_by_name(reg, out as f32)?;
                    }
                }
                'p' => {
                    let reg = self.get_register_argument(&mut source, idx);
                    let port_number =
                        self.get_number::<u8>(&mut source, idx).unwrap_or_else(|_| {
                            panic!("Value is not a valid port number ({})", idx + 1)
                        });
                    let value = match self.get_register_value(reg, idx)? {
                        RegRet::Normal(val) => val as f32,
                        RegRet::Floating(val) => val,
                    };

                    if port_number == 18 || port_number == 19 {
                        Pwm::with_frequency(
                            if port_number == 18 {
                                Channel::Pwm0
                            } else {
                                Channel::Pwm1
                            },
                            8.,
                            value as f64,
                            Polarity::Normal,
                            true,
                        )
                        .unwrap_or_else(|_| {
                            panic!("Cannot use PWM on port {port_number} ({})", idx + 1)
                        });
                    } else {
                        let mut port = self.get_port(port_number).into_output();
                        port.set_pwm_frequency(8., value as f64)
                            .unwrap_or_else(|_| {
                                panic!("Cannot use soft PWM on port {port_number} ({})", idx + 1)
                            });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_register() {
        let mut script = Angelio::from_str("lr123");
        script.run();
        assert_eq!(script.r1, 23);
    }

    #[test]
    fn basic_pid() {
        let mut script = Angelio::from_str("P2I13D7q420c69");
        script.run();
        assert_eq!(script.f3, 123553.);
    }

    #[test]
    fn add() {
        let mut script = Angelio::from_str("lr121lr237+r1r2lf13.14+r3f1");
        script.run();
        assert_eq!(script.r3, 58);
        assert_eq!(script.f3, 61.14);
    }

    #[test]
    fn move_register() {
        let mut script = Angelio::from_str("lr121lf137Tr1f1");
        script.run();
        assert_eq!(script.r1, 37);
        assert_eq!(script.f1, 21.0);
    }
}
