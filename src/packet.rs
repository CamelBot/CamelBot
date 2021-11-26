// jkcoxson

pub struct Packet {
    pub source: String,
    pub destination: String,
    pub event: String,
    pub data: String,
    pub sniffers: Vec<String>,
}

impl Clone for Packet {
    fn clone(&self) -> Packet {
        Packet {
            source: self.source.clone(),
            destination: self.destination.clone(),
            event: self.event.clone(),
            data: self.data.clone(),
            sniffers: self.sniffers.clone(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SnifferPacket {
    pub type_: String,
    pub source: String,
    pub destination: String,
    pub event: String,
    pub sniffers: Vec<String>,
    pub packet: String,
}
