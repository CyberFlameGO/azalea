//! parse sending and receiving packets with a server.

use crate::packets::game::{GameListenerTrait, GamePacket};
use crate::packets::handshake::HandshakePacket;
use crate::packets::login::LoginPacket;
use crate::packets::status::StatusPacket;
use crate::read::read_packet;
use crate::write::write_packet;
use crate::ServerIpAddress;
use tokio::net::TcpStream;

pub enum PacketFlow {
    ClientToServer,
    ServerToClient,
}

pub struct HandshakeConnection {
    pub flow: PacketFlow,
    /// The buffered writer
    pub stream: TcpStream,
}

pub struct GameConnection {
    pub flow: PacketFlow,
    /// The buffered writer
    pub stream: TcpStream,

    pub listener: Box<dyn GameListenerTrait>,
}

pub struct StatusConnection {
    pub flow: PacketFlow,
    /// The buffered writer
    pub stream: TcpStream,
}

pub struct LoginConnection {
    pub flow: PacketFlow,
    /// The buffered writer
    pub stream: TcpStream,
}

impl HandshakeConnection {
    pub async fn new(address: &ServerIpAddress) -> Result<HandshakeConnection, String> {
        let ip = address.ip;
        let port = address.port;

        let stream = TcpStream::connect(format!("{}:{}", ip, port))
            .await
            .map_err(|_| "Failed to connect to server")?;

        // enable tcp_nodelay
        stream
            .set_nodelay(true)
            .expect("Error enabling tcp_nodelay");

        Ok(HandshakeConnection {
            flow: PacketFlow::ServerToClient,
            stream,
        })
    }

    pub fn login(self) -> LoginConnection {
        LoginConnection {
            flow: self.flow,
            stream: self.stream,
        }
    }

    pub fn status(self) -> StatusConnection {
        StatusConnection {
            flow: self.flow,
            stream: self.stream,
        }
    }

    pub async fn read(&mut self) -> Result<HandshakePacket, String> {
        read_packet::<HandshakePacket>(&self.flow, &mut self.stream).await
    }

    /// Write a packet to the server
    pub async fn write(&mut self, packet: HandshakePacket) {
        write_packet(packet, &mut self.stream).await;
    }
}

impl GameConnection {
    pub async fn read(&mut self) -> Result<GamePacket, String> {
        read_packet::<GamePacket>(&self.flow, &mut self.stream).await
    }

    /// Write a packet to the server
    pub async fn write(&mut self, packet: GamePacket) {
        write_packet(packet, &mut self.stream).await;
    }

    pub fn set_listener<T: GameListenerTrait>(&mut self, listener: T) {
        self.listener = Box::new(listener);
    }
}

impl StatusConnection {
    pub async fn read(&mut self) -> Result<StatusPacket, String> {
        read_packet::<StatusPacket>(&self.flow, &mut self.stream).await
    }

    /// Write a packet to the server
    pub async fn write(&mut self, packet: StatusPacket) {
        write_packet(packet, &mut self.stream).await;
    }
}

impl LoginConnection {
    pub async fn read(&mut self) -> Result<LoginPacket, String> {
        read_packet::<LoginPacket>(&self.flow, &mut self.stream).await
    }

    /// Write a packet to the server
    pub async fn write(&mut self, packet: LoginPacket) {
        write_packet(packet, &mut self.stream).await;
    }
}
