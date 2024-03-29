use naia_bevy_shared::{
	Channel,
	ChannelDirection,
	LinkConditionerConfig,
	Protocol,
	ChannelMode,
	TickBufferSettings,
	ReliableSettings,
};
use std::time::Duration;
use super::msg;

// ~= 60fps
pub const TICK_INTERVAL: Duration = Duration::from_nanos(16_666_667);

#[derive(Channel)]
pub struct InputSrcChannel;

#[derive(Channel)]
pub struct CmdStreamChannel;

#[derive(Channel)]
pub struct EntityAssignmentChannel;

fn protocol(link_cond: Option<LinkConditionerConfig>) -> Protocol {
	let mut builder = Protocol::builder();

	if let Some(cond) = link_cond {
		builder.link_condition(cond);
	}

	builder
		.tick_interval(TICK_INTERVAL)
		.add_channel::<InputSrcChannel>(
			ChannelDirection::ClientToServer,
			ChannelMode::TickBuffered(TickBufferSettings::default()),
		)
		.add_channel::<CmdStreamChannel>(
			ChannelDirection::ServerToClient,
			ChannelMode::OrderedReliable(ReliableSettings::default())
		)
		.add_channel::<EntityAssignmentChannel>(
			ChannelDirection::ServerToClient,
			ChannelMode::UnorderedReliable(ReliableSettings::default()),
		)
		.add_message::<msg::Assign>()
		.add_message::<msg::Auth>()
		.add_message::<msg::Input>()
		.add_message::<msg::InputRepl>()
		.build()
}

pub fn perfect() -> Protocol {
	protocol(None)
}

pub fn wifi() -> Protocol {
	protocol(Some(LinkConditionerConfig::good_condition()))
}

pub fn global_avg() -> Protocol {
	protocol(Some(LinkConditionerConfig::average_condition()))
}

pub fn global_poor() -> Protocol {
	protocol(Some(LinkConditionerConfig::poor_condition()))
}
