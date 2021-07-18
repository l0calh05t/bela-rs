use bela::{AuxiliaryTask, Bela, BelaApplication, Error, RenderContext, SetupContext};

struct AuxiliaryTaskExample {
    frame_index: usize,
    tasks: Vec<AuxiliaryTask>,
}

impl AuxiliaryTaskExample {
    fn new(context: &mut SetupContext) -> Option<Self> {
        let mut tasks = Vec::new();
        let print_task = Box::new(|| {
            println!("this is a string");
        });

        let another_print_task = Box::new(|| {
            println!("this is another string");
        });

        // we solemnly promise not to reuse these names in any other process
        unsafe {
            tasks.push(
                context
                    .create_auxiliary_task(
                        print_task,
                        10,
                        &std::ffi::CString::new("printing_stuff").unwrap(),
                    )
                    .ok()?,
            );
            tasks.push(
                context
                    .create_auxiliary_task(
                        another_print_task,
                        10,
                        &std::ffi::CStr::from_bytes_with_nul(b"printing_more_stuff\0").unwrap(),
                    )
                    .ok()?,
            );
        }

        Some(Self {
            frame_index: 0,
            tasks,
        })
    }
}

unsafe impl BelaApplication for AuxiliaryTaskExample {
    fn render(&mut self, context: &mut RenderContext) {
        if self.frame_index % 1024 == 0 {
            for task in self.tasks.iter() {
                // explicitly ignore result instead of unwrapping, as unwinding here
                // is forbidden
                // TODO: find out if panic_abort would be ok
                let _ = context.schedule_auxiliary_task(task);
            }
        }

        self.frame_index = self.frame_index.wrapping_add(1);
    }
}

fn main() -> Result<(), Error> {
    Bela::new(AuxiliaryTaskExample::new).run()
}
