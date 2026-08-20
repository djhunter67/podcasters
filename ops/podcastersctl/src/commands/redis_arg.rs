use crate::redis::{self, RedisIntegration, RedisSession, RedisSubcommand};

pub fn execute(redis_arg: &redis::RedisState) {
    match &redis_arg.redis {
        RedisSubcommand::Check => {
            println!("Compare the keys in the cache with the expected values");
        }
        RedisSubcommand::Status => {
            println!("Check the status of the cache layer; uptime, number of keys, version");
        }
        RedisSubcommand::Keys(key) => match key.session {
            RedisSession::Session(ref val) => {
                println!("Get the keys for the session passed in?; Val: {val:#?}");
            }
        },
        RedisSubcommand::Clear(clear) => match clear.integration {
            RedisIntegration::IntegrationTest => {
                println!("Clearing the integration test results");
            }
        },
    }
}
