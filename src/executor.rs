use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use noop_waker::noop_waker;

pub struct Executor {
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor { tasks : Vec::new() }
    }

    pub fn spawn<F>(&mut self, future:F )
        where F: Future<Output = ()> + 'static,
    {
        self.tasks.push(Box::pin(future));
    }

    pub fn poll_all(&mut self) {
        let waker = noop_waker();
        let mut ctx = Context::from_waker(&waker);

        // 内部にあるタスクを全部１回ずつ実行して新しいベクタに入れ替えます
        // 実装的にはあんまり効率よくない気がする
        let mut next = vec!{};
        for mut task in self.tasks.drain(..) {
            if task.as_mut().poll(&mut ctx).is_pending() {
                next.push(task)
            }
        }
        self.tasks = next;
    }
}
