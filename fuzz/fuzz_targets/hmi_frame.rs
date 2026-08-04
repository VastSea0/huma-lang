#![no_main]

use huma_hmi::{read_frame, Request, Response};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut request_input = data;
    if let Ok(request) = read_frame::<Request>(&mut request_input) {
        let _ = request.validate();
    }

    let mut response_input = data;
    if let Ok(response) = read_frame::<Response>(&mut response_input) {
        let _ = response.validate();
    }
});
