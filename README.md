# Angelio

(Esoteric?) programming language with with scripting API, written in rust. It has built-in GPIO support on the Raspberry Pi.

## Usage

To call a script from a file, you can use the Angelio::new function:

```rust
let mut script = Angelio::new("path/to/file.aio");
```


Alternatively, you can create a script reading a string directly from a variable:

```rust
let mut script = Angelio::from_str("lr121lr237+r1r2+Tr3r1");    // or from_string for String
```


Then you can run the script:

```rust
script.run();
```


## Specification

Angelio has 8 number registers: 4 for integer type [r1-r4] and 4 for floating point type [f1-f4].

Here is a list of commands that the interpreter accepts:
* `l[reg][val]` - reads the value into register
* `T[reg][reg]` - swap values between registers
* `![reg]` - print value from register (with a new line)
* `o[reg][val]` - set GPIO pin `val` to the register value (0 - low, 1 - high)
* `i[reg][val]` - load value of GPIO pin `val` into register
* `p[reg][val]` - set PWM on GPIO pin `val` to register value (-1.0 - 1.0)
* `s[reg][val]` - set the servo position on GPIO pin `val` to register value (-1.0 - 1.0)
* `P[val]` - set P in PID to `val`
* `I[val]` - set I in PID to `val`
* `D[val]` - set D in PID to `val`
* `q[val]` - change the PID setpoint to `val`
* `c[val]` - calculate the PID using `val` as the measured value


## License

Angelio is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.