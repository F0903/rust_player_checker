use std::cell::RefCell;
use std::convert::TryInto;
use std::ffi::CString;
use std::io::{Error, ErrorKind, Result};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

const A2S_PLAYER_HEADER: u8 = 0x55;
const A2S_PLAYER_RESPONSE_HEADER: u8 = 0x44;

const CHALLENGE_RESPONSE_HEADER: u8 = 0x41;

pub struct Player {
	name: CString,
	duration: Duration,
}

impl Player {
	pub fn get_name(&self) -> Result<&str> {
		let st = self.name.to_str().expect("Could not convert CStr to str.");
		Ok(&st[..st.len() - 1])
	}
}

use std::fmt::{self, Display, Formatter};
impl Display for Player {
	fn fmt(&self, form: &mut Formatter<'_>) -> fmt::Result {
		form.write_str(&format!(
			"{} [{}min]",
			self.get_name().unwrap(),
			self.duration.as_secs() / 60
		))?;
		Ok(())
	}
}

pub struct Queryer {
	udp: UdpSocket,
	challenge_num: RefCell<i32>,
}

impl Queryer {
	pub fn new(local_addr: &str) -> Result<Queryer> {
		let udp = UdpSocket::bind(local_addr)?;
		let challenge_num = RefCell::new(-1);
		Ok(Queryer { udp, challenge_num })
	}

	fn send_packet(&self, to: &SocketAddr, header: u8) -> Result<()> {
		let mut packet = vec![255, 255, 255, 255];
		packet.push(header);
		packet.extend_from_slice(&self.challenge_num.borrow().to_le_bytes());
		self.udp.send_to(&packet, to)?;
		Ok(())
	}

	pub fn get_players(&self, server_addr: &SocketAddr) -> Result<Vec<Player>> {
		self.send_packet(server_addr, A2S_PLAYER_HEADER)?;
		let mut buf = [0; 10 * 1024];
		let (read, addr) = self.udp.recv_from(&mut buf)?;
		if addr != *server_addr {
			println!("Received packet from unknown server. Trying again...");
			return self.get_players(server_addr);
		}

		if read < 1 {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"Read was less than 1. If this keep occuring, please contact developer.",
			));
		}

		// Ignore first 4 bytes.
		let header = u8::from_le_bytes(buf[4..5].try_into().unwrap());
		if header == CHALLENGE_RESPONSE_HEADER {
			*self.challenge_num.borrow_mut() = i32::from_le_bytes(
				buf[5..9]
					.try_into()
					.expect("Could not convert slice to array."),
			);
			return self.get_players(server_addr);
		}
		if header != A2S_PLAYER_RESPONSE_HEADER {
			return Err(Error::new(
						ErrorKind::InvalidData,
						"A2S_PLAYERS response header was incorrect, please contact developer if this persists.",
					));
		}

		let player_count = u8::from_le_bytes(buf[5..6].try_into().unwrap());
		let mut players = Vec::<Player>::with_capacity(player_count as usize);
		let mut player_offset = 6;
		for _num in 0..player_count {
			// Ignore index.
			let index_offset = player_offset;
			let index_length = 1;

			let name_offset = index_offset + index_length;

			let mut name = Vec::<u8>::with_capacity(15);
			for cha in buf[name_offset..].iter() {
				name.push(*cha);
				if cha == &b'\0' {
					break;
				}
			}

			// Ignore score.
			let score_offset = name_offset + name.len();
			let score_length = 4;

			let duration_offset = score_offset + score_length;
			let duration_length = 4;
			let duration = f32::from_le_bytes(
				buf[duration_offset..duration_offset + duration_length]
					.try_into()
					.unwrap(),
			);

			player_offset = duration_offset + duration_length;

			players.push(Player {
				name: unsafe { CString::from_vec_unchecked(name) },
				duration: Duration::from_secs_f32(duration),
			});
		}
		Ok(players)
	}
}
