pub struct PID {
    pub p: f32,
    pub i: f32,
    pub d: f32,
    pub setpoint: f32,
    prev_err: f32,
    tot_err: f32,
}

impl PID {
    pub fn new(p: f32, i: f32, d: f32) -> PID {
        PID {
            p,
            i,
            d,
            setpoint: 0.0,
            prev_err: 0.0,
            tot_err: 0.0,
        }
    }

    pub fn calculate(&mut self, measurement: f32) -> f32 {
        let pos_err = self.setpoint - measurement;
        let verr = (pos_err - self.prev_err) / 0.02;
        self.prev_err = pos_err;
        if self.i != 0.0 {
            let a = self.tot_err + pos_err * 0.02;
            if a < -1.0 / self.i {
                self.tot_err = -1.0 / self.i;
            } else if a > 1.0 / self.i {
                self.tot_err = 1.0 / self.i;
            } else {
                self.tot_err = a;
            }
        }
        self.p * pos_err + self.i * self.tot_err + self.d * verr
    }
}
