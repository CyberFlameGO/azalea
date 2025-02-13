use azalea_buf::McBuf;
use uuid::Uuid;

#[derive(Debug, Clone, McBuf)]
pub struct SaltSignaturePair {
    pub salt: u64,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Default, McBuf)]
pub struct MessageSignature {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, McBuf)]
pub struct SignedMessageHeader {
    pub previous_signature: Option<MessageSignature>,
    pub sender: Uuid,
}
