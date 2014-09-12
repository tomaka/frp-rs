#![feature(tuple_indexing)]

extern crate frp;
extern crate time;

struct Movement;
impl frp::Property<(f32, f32)> for Movement {}

struct Position;
impl frp::Property<(f32, f32)> for Position {}

struct Mario;
impl frp::Property<frp::EntityID> for Mario {}

struct Clock;
impl frp::Property<f32> for Clock {}

fn main() {
    // initializing the state
    let state = init();

    // main loop
    let mut timer = std::io::timer::Timer::new().unwrap();
    let periodic = timer.periodic(std::time::Duration::milliseconds(16));

    loop {
        // printing mario's position
        println!("{}", state.entity(&state.get(Mario).unwrap()).unwrap().get(Position));

        // waiting for next iteration
        periodic.recv();
    }
}

fn init() -> frp::State {
    let mut state = frp::State::new();

    // adding Clock
    {
        fn get_clock(_: &frp::State, _: Option<frp::Entity>) -> f32 {
            (time::precise_time_ns() as f32) / 1000000000.0
        }

        state.add(Clock, frp::behaviors::Alias(get_clock));   
    }

    // adding Mario
    state.add(Mario, frp::behaviors::Storage(None, get_mario));

    state
}

fn get_mario(mut current: &mut Option<frp::EntityID>, state: &frp::State, _: Option<frp::Entity>) -> frp::EntityID {
    if current.is_some() {  // TODO: use if let syntax
        return current.unwrap();
    }

    // creating Mario
    let mut entity = state.create_entity();

    // adding Movement
    entity.add(Movement, frp::behaviors::Constant((1.0f32, 0.5f32)));

    // adding Position
    {
        fn update_position(&(ref mut x, ref mut y, ref mut time): &mut (f32, f32, f32), state: &frp::State, me: Option<frp::Entity>) -> (f32, f32) {
            let clock = state.get(Clock).unwrap();
            let clock_diff = clock - *time;
            *time = clock;

            let mv = me.unwrap().get(Movement).unwrap_or((0.0, 0.0));
            *x += mv.0 * clock_diff;
            *y += mv.1 * clock_diff;

            (*x, *y)
        }

        let clock = state.get(Clock).unwrap();
        entity.add(Position, frp::behaviors::Storage((0.0f32, 0.0f32, clock), update_position));
    }

    *current = Some(entity.get_id());
    entity.get_id()
}
