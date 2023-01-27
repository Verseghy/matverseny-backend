use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(not(unix))]
use futures::future::BoxFuture;
#[cfg(unix)]
use tokio::signal::unix::Signal;

pub struct SignalHandler {
    #[cfg(unix)]
    signals: smallvec::SmallVec<[Signal; 3]>,
    #[cfg(not(unix))]
    signals: BoxFuture<'static, Result<()>>,
}

impl SignalHandler {
    pub fn new() -> Self {
        trace!("registering signal listeners");

        #[cfg(unix)]
        {
            use tokio::signal::unix::{self, SignalKind};

            let signals = [
                SignalKind::interrupt(),
                SignalKind::terminate(),
                SignalKind::quit(),
            ];

            let signals = signals
                .into_iter()
                .filter_map(|signal| {
                    unix::signal(signal)
                        .map_err(|error| {
                            error!(
                                "failed to initialize signal listener: {:?}. error: {}",
                                signal, error
                            );
                        })
                        .ok()
                })
                .collect();

            SignalHandler { signals }
        }

        #[cfg(not(unix))]
        {
            use tokio::signal::ctrl_c;

            SignalHandler {
                signals: Box::pin(ctrl_c()),
            }
        }
    }
}

impl Future for SignalHandler {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[cfg(unix)]
        {
            for signal in &mut self.signals {
                if signal.poll_recv(cx).is_ready() {
                    return Poll::Ready(());
                }
            }
        }

        #[cfg(not(unix))]
        {
            if self.signals.as_mut().poll(cx).is_ready() {
                return Poll::Ready(());
            }
        }

        Poll::Pending
    }
}
