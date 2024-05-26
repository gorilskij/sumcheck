use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};

use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

pub type Buf = Vec<u8>;

pub struct Channel(Sender<Buf>, Receiver<Buf>);

impl Channel {
    pub fn new_pair() -> (Self, Self) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        (Self(tx1, rx2), Self(tx2, rx1))
    }

    fn raw_send(&self, buf: Buf) -> Result<(), SendError<Buf>> {
        self.0.send(buf)
    }

    fn raw_recv(&self) -> Result<Buf, RecvError> {
        self.1.recv()
    }

    pub fn send<T: CanonicalSerialize>(&self, data: T) -> Result<()> {
        let mut buf = vec![];
        data.serialize_compressed(&mut buf)?;
        self.raw_send(buf)?;
        Ok(())
    }

    pub fn recv<T: CanonicalDeserialize>(&self) -> Result<T> {
        let buf = self.raw_recv()?;
        let value = T::deserialize_compressed(&*buf)?;
        Ok(value)
    }
}
