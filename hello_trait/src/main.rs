#[derive(Clone)]
pub struct StepIO {
    pub steps: usize,
}

#[derive(Clone)]
pub struct JumpIO {
    pub jumps: usize,
}

#[derive(Clone)]
pub enum IO {
    Steps(StepIO),
    Jumps(JumpIO),
}

impl From<JumpIO> for IO {
    fn from(jump_io: JumpIO) -> Self {
        IO::Jumps(jump_io)
    }
}

impl From<StepIO> for IO {
    fn from(step_io: StepIO) -> Self {
        IO::Steps(step_io)
    }
}

impl From<StepIO> for JumpIO {
    fn from(step_io: StepIO) -> Self {
        Self {
            jumps: step_io.steps / 2,
        }
    }
}

impl From<IO> for JumpIO {
    fn from(io: IO) -> Self {
        match io {
            IO::Jumps(jump_io) => jump_io,
            IO::Steps(step_io) => step_io.into(),
        }
    }
}

impl From<JumpIO> for StepIO {
    fn from(jump_io: JumpIO) -> Self {
        Self {
            steps: jump_io.jumps * 2,
        }
    }
}

impl From<IO> for StepIO {
    fn from(io: IO) -> Self {
        match io {
            IO::Jumps(jump_io) => jump_io.into(),
            IO::Steps(step_io) => step_io,
        }
    }
}

pub trait WalkerOrJumper<T> {
    fn walk_or_jump(&self, t: T) -> T;
}

pub trait Walker {
    fn walk(&self, io: StepIO) -> StepIO;
}

pub trait Jumper {
    fn jump(&self, io: JumpIO) -> JumpIO;
}

pub fn walk_or_jump<T>(instructions: &[Box<T>], step_io: StepIO) -> StepIO
where
    T: WalkerOrJumper<IO> + ?Sized,
{
    if instructions.is_empty() {
        return step_io;
    }

    let initial_io = IO::Steps(step_io);

    let final_io = instructions
        .iter()
        .fold(initial_io, |io, next_walker_or_jumper| {
            next_walker_or_jumper.walk_or_jump(io)
        });

    match final_io {
        IO::Steps(step_io) => step_io,
        IO::Jumps(jump_io) => {
            let step_io: StepIO = jump_io.into();
            step_io
        }
    }
}

struct WalkerWrapper<W>(W);
struct JumperWrapper<J>(J);

impl<T> WalkerOrJumper<IO> for WalkerWrapper<T>
where
    T: Walker,
{
    fn walk_or_jump(&self, io: IO) -> IO {
        self.0.walk(io.into()).into()
    }
}

impl<T> WalkerOrJumper<IO> for JumperWrapper<T>
where
    T: Jumper,
{
    fn walk_or_jump(&self, io: IO) -> IO {
        self.0.jump(io.into()).into()
    }
}

fn main() {
    pub struct ExactWalker {
        steps_at_once: usize,
    }

    pub struct ExactJumper {
        jumps_at_once: usize,
    }

    impl Walker for ExactWalker {
        fn walk(&self, io: StepIO) -> StepIO {
            println!("Walking {} steps", io.steps);
            StepIO {
                steps: io.steps - self.steps_at_once,
            }
        }
    }

    impl Jumper for ExactJumper {
        fn jump(&self, io: JumpIO) -> JumpIO {
            println!("Jumping {} times", io.jumps);
            JumpIO {
                jumps: io.jumps - self.jumps_at_once,
            }
        }
    }

    let instructions: Vec<Box<WalkerOrJumper<IO>>> = vec![
        Box::new(WalkerWrapper(ExactWalker { steps_at_once: 10 })),
        Box::new(JumperWrapper(ExactJumper { jumps_at_once: 20 })),
    ];

    let io = StepIO { steps: 300 };

    let final_io = walk_or_jump(&instructions, io.clone());
    let final_io_jumps: JumpIO = final_io.clone().into();

    println!(
        "steps/jumps left: {}/{}",
        final_io.steps, final_io_jumps.jumps
    );
}
