#![feature(tuple_indexing)]

extern crate frp;
extern crate time;

struct Movement;
impl frp::Property<(f32, f32)> for Movement {}

struct Position;
impl frp::Property<(f32, f32)> for Position {}

struct Clock;
impl frp::Property<f32> for Clock {}

fn main() {
    let mut state = frp::State::new();

    // adding Movement
    state.add(Movement, frp::behaviors::Constant((1.0f32, 0.5f32)));

    // adding Clock
    {
        fn get_clock(_: &frp::State) -> f32 {
            (time::precise_time_ns() as f32) / 1000000000.0
        }

        state.add(Clock, frp::behaviors::Alias(get_clock));   
    }

    // adding Position
    {
        fn update_position(&(mut x, mut y, mut time): &mut (f32, f32, f32), state: &frp::State) -> (f32, f32) {
            let clock = state.get(Clock).unwrap();
            let clock_diff = clock - time;
            time = clock;

            let mv = state.get(Movement).unwrap_or((0.0, 0.0));
            x += mv.0 * clock_diff;
            y += mv.1 * clock_diff;

            (x, y)
        }

        let clock = state.get(Clock).unwrap();
        state.add(Position, frp::behaviors::Storage((0.0f32, 0.0f32, clock), update_position));
    }

    // main loop
    let mut timer = std::io::timer::Timer::new().unwrap();
    let periodic = timer.periodic(std::time::Duration::milliseconds(16));
    loop {
        println!("{}", state.get(Position));
        periodic.recv();
    }
}
