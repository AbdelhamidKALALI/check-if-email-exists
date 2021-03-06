// check-if-email-exists
// Copyright (C) 2018-2020 Amaury Martiny

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// ref: https://github.com/amaurymartiny/check-if-email-exists/issues/568
#![type_length_limit = "2097152000000000"]

extern crate clap;
extern crate env_logger;
extern crate hyper;
extern crate serde;
extern crate tokio;

mod http;

use check_if_email_exists::{check_email, CheckEmailInput};
use clap::{crate_version, value_t, App};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	// The YAML file is found relative to the current file, similar to how modules are found
	let yaml = clap::load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

	if let Some(to_email) = matches.value_of("TO_EMAIL") {
		let from_email = matches
			.value_of("FROM_EMAIL")
			.expect("FROM_EMAIL has a default value. qed.");
		let hello_name = matches
			.value_of("HELLO_NAME")
			.expect("HELLO_NAME has a default value. qed.");

		let mut input = CheckEmailInput::new(vec![to_email.into()]);
		input
			.from_email(from_email.into())
			.hello_name(hello_name.into());

		if let Some(proxy_host) = matches.value_of("PROXY_HOST") {
			let proxy_port = value_t!(matches.value_of("PROXY_PORT"), u16)
				.expect("PROXY_PORT has a default value of type u16. qed.");
			input.proxy(proxy_host.into(), proxy_port);
		}

		if let Ok(yahoo_use_api) = value_t!(matches.value_of("YAHOO_USE_API"), bool) {
			input.yahoo_use_api(yahoo_use_api);
		}

		let result = check_email(&input).await;

		match serde_json::to_string_pretty(&result) {
			Ok(output) => {
				println!("{}", output);
			}
			Err(err) => {
				println!("{}", err);
			}
		};
	}

	// Run the web server if flag is on
	if matches.is_present("HTTP") {
		let http_host = matches
			.value_of("HTTP_HOST")
			.expect("HTTP_HOST has a default value. qed.");
		// http_port is, in this order:
		// - the value of `--http-port` flag
		// - if not set, then the $PORT env varialbe
		// - if not set, then 3000
		let env_port = env::var("PORT").unwrap_or_else(|_| "3000".into());
		let http_port = matches.value_of("HTTP_PORT").unwrap_or(&env_port);

		http::run(http_host, http_port.parse::<u16>().unwrap()).await?
	}

	Ok(())
}
