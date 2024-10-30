use anyhow::Result;

pub trait TwitchEventHandler {
    fn subscribed_events(&self) -> &[twitcheventsub::Subscription];
    fn handle_event(
        &mut self,
        event: &twitcheventsub::Event,
        api: &mut twitcheventsub::TwitchEventSubApi,
    ) -> Result<()>;
}
