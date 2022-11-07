mod sync_server;
mod async_server;

use sync_server::{
    run,
    utils::{
        types::ServerError,
        consts::THREAD_POOL_SIZE,
    },
};


fn main() -> Result<(), ServerError> {
    run(THREAD_POOL_SIZE)
}
