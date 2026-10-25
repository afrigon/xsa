use std::time::Duration;

use anyhow::{Context, anyhow, ensure};
use tokio::sync::oneshot;
use xsa_commands::value::CameraId;
use xsa_proto::error::RequestError;
use xsa_proto::frame::RequestId;
use xsa_proto::reply::Reply;
use xsa_proto::request::{Request, SetTimeRateOverrideRequest, StepTimeRequest};
use xsa_units::{SimulationDuration, TimeRate};

use super::App;
use super::capture_end::CaptureEnd;
use super::capture_phase::CapturePhase;
use super::frame_pace::FramePace;
use super::video_capture::VideoCapture;

impl App {
    pub(super) fn start_video_capture(
        &mut self,
        camera: CameraId,
        fps: u32,
        end: CaptureEnd,
        completion: Option<oneshot::Sender<anyhow::Result<String>>>,
    ) -> anyhow::Result<()> {
        ensure!(fps > 0, "a capture needs at least 1 frame per second");
        ensure!(
            !self.captures.contains_key(&camera),
            "camera {camera} is already capturing"
        );
        let state = self.session.state().context("the session has not started")?;
        let request = Request::SetTimeRateOverride(SetTimeRateOverrideRequest {
            rate: Some(TimeRate::PAUSED),
        });
        request
            .check_session_environment(state.environment())
            .map_err(|error| anyhow!("{error}"))?;
        let previous_override = state.time_flow().time_override;
        let id = self.session.send(request)?;
        self.captures.insert(
            camera,
            VideoCapture {
                fps,
                frames: 0,
                end,
                phase: CapturePhase::Starting,
                awaiting: Some(id),
                previous_override,
                completion,
            },
        );
        self.follow_capture_pace();

        Ok(())
    }

    pub(super) fn stop_video_capture(
        &mut self,
        camera: &CameraId,
    ) -> anyhow::Result<oneshot::Receiver<anyhow::Result<String>>> {
        let capture = self
            .captures
            .get_mut(camera)
            .with_context(|| format!("camera {camera} is not capturing"))?;
        capture.end = CaptureEnd::AfterFrames { frames: capture.frames };
        let (completion, receiver) = oneshot::channel();
        capture.completion = Some(completion);

        Ok(receiver)
    }

    // A frame waits while a capture waits on the server, so it shows the time the previous step reached.
    pub(super) fn frame_ready(&self) -> bool {
        self.captures.values().all(|capture| capture.awaiting.is_none())
    }

    pub(super) fn handle_capture_reply(&mut self, id: RequestId, result: &Result<Reply, RequestError>) {
        let Some(camera) = self
            .captures
            .iter()
            .find(|(_, capture)| capture.awaiting == Some(id))
            .map(|(camera, _)| camera.clone())
        else {
            return;
        };

        if let Err(error) = result {
            self.fail_capture(&camera, anyhow!("{error}"));

            return;
        }

        let Some(capture) = self.captures.get_mut(&camera) else {
            return;
        };
        capture.awaiting = None;

        match capture.phase {
            CapturePhase::Starting => capture.phase = CapturePhase::Running,
            CapturePhase::Running if capture.is_done() => self.finish_capture(&camera),
            CapturePhase::Running => {}
            CapturePhase::Finishing => {
                let summary = format!("captured {} frames at {} fps", capture.frames, capture.fps);
                capture.complete(Ok(summary));
                self.captures.remove(&camera);
                self.follow_capture_pace();
            }
        }
    }

    // After each frame, the last one included: counts it and steps time by one frame's worth, so a capture of n
    // frames leaves time n steps later. The capture finishes once the last step is in.
    pub(super) fn advance_captures(&mut self) {
        let Some(rate) = self.session.state().map(|state| state.time_flow().rate) else {
            return;
        };
        let cameras: Vec<CameraId> = self.captures.keys().cloned().collect();

        for camera in cameras {
            let Some(capture) = self.captures.get_mut(&camera) else {
                continue;
            };

            if capture.phase != CapturePhase::Running || capture.awaiting.is_some() {
                continue;
            }

            capture.frames += 1;
            let step = rate.scale(SimulationDuration {
                seconds: 1.0 / f64::from(capture.fps),
            });

            if !step.is_valid_step() {
                if capture.is_done() {
                    self.finish_capture(&camera);
                }

                continue;
            }

            match self.session.send(Request::StepTime(StepTimeRequest { duration: step })) {
                Ok(id) => capture.awaiting = Some(id),
                Err(err) => self.fail_capture(&camera, err),
            }
        }
    }

    fn finish_capture(&mut self, camera: &CameraId) {
        let Some(capture) = self.captures.get_mut(camera) else {
            return;
        };
        capture.phase = CapturePhase::Finishing;
        let restore = Request::SetTimeRateOverride(SetTimeRateOverrideRequest {
            rate: capture.previous_override,
        });

        match self.session.send(restore) {
            Ok(id) => capture.awaiting = Some(id),
            Err(err) => self.fail_capture(camera, err),
        }
    }

    fn fail_capture(&mut self, camera: &CameraId, error: anyhow::Error) {
        let Some(mut capture) = self.captures.remove(camera) else {
            return;
        };

        if capture.phase != CapturePhase::Finishing {
            let restore = Request::SetTimeRateOverride(SetTimeRateOverrideRequest {
                rate: capture.previous_override,
            });

            if let Err(err) = self.session.send(restore) {
                tracing::warn!("giving time back after a failed capture: {err:#}");
            }
        }

        capture.complete(Err(error.context(format!("capturing camera {camera}"))));
        self.follow_capture_pace();
    }

    fn follow_capture_pace(&mut self) {
        let pace = match self.captures.values().next() {
            Some(capture) => FramePace::Fixed {
                step: Duration::from_secs_f64(1.0 / f64::from(capture.fps)),
            },
            None => FramePace::Wall,
        };
        self.clock.set_pace(pace);
    }
}
