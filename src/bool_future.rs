use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

unsafe fn rwwake(_p: *const ()) {}

unsafe fn rwwakebyref(_p: *const ()) {}

unsafe fn rwdrop(_p: *const ()) {}

static VTABLE: RawWakerVTable = RawWakerVTable::new(rwclone, rwwake, rwwakebyref, rwdrop);

fn noop_waker() -> RawWaker {
    static DATA: () = ();
    RawWaker::new(&DATA, &VTABLE)
}

unsafe fn rwclone(_p: *const ()) -> RawWaker {
    noop_waker()
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    pin_utils::pin_mut!(future);
    let waker = &unsafe { Waker::from_raw(noop_waker()) };
    let mut cx = Context::from_waker(waker);

    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
    }
}

pub struct BoolFuture<F: Fn() -> bool>(pub F);

impl<F: Fn() -> bool> Future for BoolFuture<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0() {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
