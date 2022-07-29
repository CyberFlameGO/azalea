use azalea_buf::McBuf;
use azalea_chat::component::Component;
use packet_macros::GamePacket;

#[derive(Clone, Debug, McBuf, GamePacket)]
pub struct ClientboundSystemChatPacket {
    pub content: Component,
    pub overlay: bool,
}
