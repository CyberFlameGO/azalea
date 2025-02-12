use azalea_buf::McBuf;
use azalea_protocol_macros::ClientboundGamePacket;

#[derive(Clone, Debug, McBuf, ClientboundGamePacket)]
pub struct ClientboundPlayerCombatEndPacket {
    #[var]
    pub duration: u32,
    pub killer_id: u32,
}
