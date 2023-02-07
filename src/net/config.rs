use naia_shared::{Channel, SharedConfig, LinkConditionerConfig, SocketConfig};
use super::protocol::Channels;
use std::time::Duration;

const CHANNEL_CONFIG: &[Channel<Channels>] = &[];

// ~= 60fps
const TICK_INTERVAL: Duration = Duration::from_nanos(16_666_667);

fn shared_config(link_condition: Option<LinkConditionerConfig>) -> SharedConfig<Channels> {
	SharedConfig::new(
		SocketConfig::new(link_condition, None),
		CHANNEL_CONFIG,
		Some(TICK_INTERVAL),
		None,
	)
}

pub fn perfect() -> SharedConfig<Channels> {
	shared_config(None)
}

pub fn wifi() -> SharedConfig<Channels> {
	shared_config(Some(LinkConditionerConfig::good_condition()))
}

pub fn global_avg() -> SharedConfig<Channels> {
	shared_config(Some(LinkConditionerConfig::average_condition()))
}

pub fn global_poor() -> SharedConfig<Channels> {
	shared_config(Some(LinkConditionerConfig::poor_condition()))
}
