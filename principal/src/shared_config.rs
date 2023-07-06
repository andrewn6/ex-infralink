use reqwest::Client;

pub struct SharedConfig {
	pub clients: ProviderClients,
}

pub struct ProviderClients {
	pub vultr: Option<Client>,
	pub hetzner: Option<Client>,
	pub oracle: Option<Client>,
}

impl ProviderClients {
	pub fn vultr(&mut self) -> &Client {
		if self.vultr.is_none() {
			self.vultr = Some(Client::builder().use_rustls_tls().build().unwrap());
		} 

		self.vultr.as_ref().unwrap()
	}

	pub fn hetzner(&mut self) -> &Client {
		if self.hetzner.is_none() {
			self.hetzner = Some(Client::builder().use_rustls_tls().build().unwrap());
		}

		self.hetzner.as_ref().unwrap()
	}


	pub fn oracle(&mut self) -> &Client {
		if self.oracle.is_none() {
			self.oracle = Some(Client::builder().use_rustls_tls().build().unwrap());
		}

		self.oracle.as_ref().unwrap()
	}
}
