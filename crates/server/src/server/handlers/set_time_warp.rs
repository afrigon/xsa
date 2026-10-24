use xsa_proto::error::RequestError;
use xsa_proto::request::SetTimeWarpRequest;
use xsa_units::WarpPace;

use crate::server::Server;
use crate::server::request_handler::RequestHandler;

impl RequestHandler for SetTimeWarpRequest {
    fn handle(self, server: &mut Server, _client: usize) -> Result<(), RequestError> {
        if let Some(warp) = self.warp {
            if !warp.until.is_valid() || warp.until <= server.world.time() {
                return Err(RequestError::invalid(
                    "a warp ends at a finite time after the current one",
                ));
            }

            if let WarpPace::Eased { seconds } = warp.pace
                && !(seconds.is_finite() && seconds > 0.0)
            {
                return Err(RequestError::invalid(
                    "an eased warp lasts a finite number of seconds above 0",
                ));
            }
        }

        server.world.set_warp(self.warp);
        server.broadcast_time();

        Ok(())
    }
}
