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
}

impl Queryer {
	pub fn new(local_addr: &str) -> Result<Queryer> {
		let udp = UdpSocket::bind(local_addr)?;
		Ok(Queryer { udp })
	}

	fn get_challenge_num(&self, header: u8, server_addr: &SocketAddr) -> Result<i32> {
		let mut packet = vec![255, 255, 255, 255]; // Needs these 4 bytes for some reason.
		packet.push(header);
		packet.extend_from_slice(&(-1i32).to_le_bytes());
		self.udp
			.send_to(&packet, server_addr)
			.expect("Could not send packet.");
		let mut buf = [0; 1400];
		loop {
			if let Ok((read, addr)) = self.udp.recv_from(&mut buf) {
				if addr != *server_addr {
					println!("Received packet from unknown server.");
					continue;
				}

				if read < 1 {
					panic!("Read was less than 1.");
				}

				//Ignore first 4 bytes.
				let header = u8::from_le_bytes(
					buf[4..5]
						.try_into()
						.expect("Could not convert slice to array."),
				);
				if header != CHALLENGE_RESPONSE_HEADER {
					panic!("Header response was not correct.");
				}

				let chall_num = i32::from_le_bytes(
					buf[5..9]
						.try_into()
						.expect("Could not convert slice to array."),
				);
				return Ok(chall_num);
			} else {
				panic!("Error occurred while receiving packet.");
			}
		}
	}

	pub fn get_players(&self, server_addr: &SocketAddr) -> Result<Vec<Player>> {
		let challenge_num = self
			.get_challenge_num(A2S_PLAYER_HEADER, server_addr)
			.expect("err");
		let mut packet = vec![255, 255, 255, 255];
		packet.push(A2S_PLAYER_HEADER);
		packet.extend_from_slice(&challenge_num.to_le_bytes());
		self.udp
			.send_to(&packet, server_addr)
			.expect("Could not send.");

		let mut buf = [0; 10 * 1024];
		loop {
			let res = self.udp.recv_from(&mut buf);
			if let Ok((read, addr)) = res {
				if addr != *server_addr {
					println!("Received packet from unknown server. Trying again...");
					continue;
				}

				if read < 1 {
					return Err(Error::new(
						ErrorKind::InvalidData,
						"Read was less than 1. If this keep occuring, please contact developer.",
					));
				}

				// Ignore first 4 bytes.
				let header = u8::from_le_bytes(buf[4..5].try_into().unwrap());
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
				return Ok(players);
			} else {
				return Err(Error::new(
					ErrorKind::InvalidData,
					format!(
						"Error occurred during A2S_PLAYERS response:\n {}",
						res.unwrap_err()
					),
				));
			}
		}
	}
}
